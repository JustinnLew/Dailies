use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

use crate::state::trivia::{TriviaQuestion, TriviaStyle};

// ---------------------------------------------------------------------------
// Subtopic cache
// ---------------------------------------------------------------------------

/// Default maximum number of subtopics held across all topics combined.
const CACHE_CAPACITY: usize = 150;
const LOG_LLM_OUTPUT: bool = false;

/// Path to the persisted cache file, overridable via env var.
fn cache_path() -> PathBuf {
    env::var("SUBTOPIC_CACHE_PATH")
        .unwrap_or_else(|_| "subtopic_cache.json".to_string())
        .into()
}

/// Persistent, globally-shared cache of recently used subtopics.
#[derive(Serialize, Deserialize, Default)]
struct SubtopicCache {
    seen: HashMap<String, Vec<String>>,
    capacity: usize,
}

impl SubtopicCache {
    fn new(capacity: usize) -> Self {
        Self {
            seen: HashMap::new(),
            capacity,
        }
    }

    fn total_len(&self) -> usize {
        self.seen.values().map(|v| v.len()).sum()
    }

    fn insert(&mut self, topic: &str, subtopic: String) {
        let bucket = self.seen.entry(topic.to_string()).or_default();
        if bucket.contains(&subtopic) {
            return;
        }

        if self.total_len() >= self.capacity {
            let mut rng = rand::rng();
            let victim_topic = self.seen.keys().choose(&mut rng).cloned();

            if let Some(vt) = victim_topic {
                let bucket = self.seen.get_mut(&vt).unwrap();
                if !bucket.is_empty() {
                    let idx = (0..bucket.len()).choose(&mut rng).unwrap();
                    let evicted = bucket.swap_remove(idx);
                    info!(topic=%vt, subtopic=%evicted, "Evicted subtopic from cache");
                }
                // Clean up empty buckets so the map doesn't grow forever.
                if self.seen.get(&vt).map_or(false, |b| b.is_empty()) {
                    self.seen.remove(&vt);
                }
            }
        }

        self.seen
            .entry(topic.to_string())
            .or_default()
            .push(subtopic);
    }

    /// All known subtopics for `topic`, to be injected into the prompt
    /// as a blocklist.
    fn blocklist(&self, topic: &str) -> Vec<&str> {
        self.seen
            .get(topic)
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }
}

/// Load the cache from disk, returning an empty one on any error.
async fn load_cache() -> SubtopicCache {
    let path = cache_path();
    match tokio::fs::read_to_string(&path).await {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            warn!(path=?path, error=%e, "Failed to parse subtopic cache, starting fresh");
            SubtopicCache::new(CACHE_CAPACITY)
        }),
        Err(_) => {
            // File doesn't exist yet — that's fine on first run.
            SubtopicCache::new(CACHE_CAPACITY)
        }
    }
}

/// Persist the cache back to disk. Failures are warnings only.
async fn save_cache(cache: &SubtopicCache) {
    let path = cache_path();
    let json = match serde_json::to_string_pretty(cache) {
        Ok(s) => s,
        Err(e) => {
            warn!(error=%e, "Failed to serialise subtopic cache");
            return;
        }
    };
    match tokio::fs::write(&path, json).await {
        Ok(_) => info!(path=?path, "Subtopic cache saved"),
        Err(e) => warn!(path=?path, error=%e, "Failed to save subtopic cache"),
    }
}

// ---------------------------------------------------------------------------
// LLM output logging
// ---------------------------------------------------------------------------

/// One entry written to the log file per LLM call.
#[derive(Serialize)]
struct LlmLogEntry<'a> {
    stage: &'a str,
    model: &'a str,
    topic: &'a str,
    raw_response: &'a str,
}

/// Append one pretty-printed JSON entry to the log file, separated by a
/// blank line. Failures are warnings only — logging must not break the
/// main flow.
async fn log_llm_output(stage: &str, model: &str, topic: &str, raw: &str) {
    let log_path: PathBuf = env::var("LLM_LOG_PATH")
        .unwrap_or_else(|_| "llm_output.json".to_string())
        .into();

    let entry = LlmLogEntry {
        stage,
        model,
        topic,
        raw_response: raw,
    };

    let mut line = match serde_json::to_string_pretty(&entry) {
        Ok(s) => s,
        Err(e) => {
            warn!(error=%e, "Failed to serialise LLM log entry");
            return;
        }
    };
    line.push_str("\n\n");

    match OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .await
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(line.as_bytes()).await {
                warn!(path=?log_path, error=%e, "Failed to write LLM log entry");
            } else {
                info!(path=?log_path, stage=%stage, "LLM output logged");
            }
        }
        Err(e) => {
            warn!(path=?log_path, error=%e, "Failed to open LLM log file");
        }
    }
}

// ---------------------------------------------------------------------------
// Ollama client
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    seed: u64,
    top_k: u32,
    top_p: f32,
}

impl Default for OllamaOptions {
    fn default() -> Self {
        Self {
            temperature: 0.8,
            seed: rand::random(),
            top_k: 20,
            top_p: 0.9,
        }
    }
}

#[derive(Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

/// Shared HTTP call to Ollama — returns the raw response string.
async fn ollama_call(
    client: &reqwest::Client,
    url: &str,
    model: &str,
    prompt: String,
    options: OllamaOptions,
) -> Result<String, String> {
    let req_body = OllamaGenerateRequest {
        model: model.to_string(),
        prompt,
        stream: false,
        options,
    };

    let response = client
        .post(url)
        .json(&req_body)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Ollama connection error: {}. Make sure Ollama is running at {} and the model is pulled.",
                e, url
            )
        })?;

    if !response.status().is_success() {
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        warn!(status=?status, err=%err_text, "Ollama returned error status");
        return Err(format!(
            "Ollama server returned error {}: {}",
            status, err_text
        ));
    }

    let gen_response = response
        .json::<OllamaGenerateResponse>()
        .await
        .map_err(|e| format!("Failed to parse Ollama envelope: {}", e))?;

    Ok(gen_response.response)
}

// ---------------------------------------------------------------------------
// Generation steps
// ---------------------------------------------------------------------------

/// Step 1 — generate n well-known, distinct subtopics for the given topic,
/// excluding anything already in the cache.
async fn generate_subtopics(
    client: &reqwest::Client,
    url: &str,
    model: &str,
    topic: &str,
    num_questions: u8,
    cache: &SubtopicCache,
) -> Result<Vec<String>, String> {
    let blocklist = cache.blocklist(topic);
    let blocklist_clause = if blocklist.is_empty() {
        String::new()
    } else {
        format!(
            "The following subtopics have been used recently and must NOT appear in any form: [{}]. \
            Choose subtopics that are clearly different in theme, era, and domain from this list. ",
            blocklist.join(", ")
        )
    };

    let prompt = format!(
        "Think step by step: \
        First, brainstorm 20 diverse subtopics about '{topic}' spanning different domains, regions, and eras. \
        Then eliminate any that are among the top 5 most commonly known examples of '{topic}'. \
        Then eliminate any that share a domain or era with another remaining subtopic. \
        {blocklist_clause}\
        From what remains, select exactly {num_questions} subtopics that are maximally different from each other. \
        Finally, output ONLY a raw JSON array of your {num_questions} chosen subtopics. \
        Do not include any reasoning, preamble, or markdown in your final output."
    );

    let options = OllamaOptions {
        temperature: 1.1,
        top_k: 150,
        top_p: 0.95,
        ..Default::default()
    };

    info!(
        url=%url, model=%model, topic=%topic,
        blocklist_size=%blocklist.len(),
        "Generating subtopics"
    );

    let raw = ollama_call(client, url, model, prompt, options).await?;
    if LOG_LLM_OUTPUT {
        log_llm_output("subtopics", model, topic, &raw).await;
    }

    let subtopics: Vec<String> = serde_json::from_str(&raw).map_err(|e| {
        warn!(response=%raw, error=%e, "Failed to parse subtopics JSON");
        format!(
            "Invalid subtopics JSON from model: {}. Raw response: {}",
            e, raw
        )
    })?;

    if subtopics.len() != num_questions as usize {
        warn!(
            expected=%num_questions,
            got=%subtopics.len(),
            "Subtopic count mismatch"
        );
    }

    info!(count=%subtopics.len(), "Successfully generated subtopics");
    Ok(subtopics)
}

/// Step 2 — generate one question per subtopic.
async fn generate_questions_from_subtopics(
    client: &reqwest::Client,
    url: &str,
    model: &str,
    topic: &str,
    subtopics: &[String],
    style: TriviaStyle,
) -> Result<Vec<TriviaQuestion>, String> {
    let subtopic_list = subtopics
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {}", i + 1, s))
        .collect::<Vec<_>>()
        .join(", ");

    let prompt = match style {
        TriviaStyle::MultipleChoice => format!(
            "You are a professional trivia generator. For each of the following subtopics about '{topic}', \
            generate exactly one multiple-choice trivia question. \
            Subtopics: [{subtopic_list}]. \
            Each question MUST have exactly 4 choices, and a correct_answer which MUST be exactly one of the choices. \
            Do not prefix choices with letters like A, B, C, D. Just provide the raw text. \
            Your response must be a single raw valid JSON array with exactly {n} elements. \
            Do not wrap in markdown or any other tags. \
            The JSON schema is: \
            [ \
              {{ \
                \"question\": \"Question text?\", \
                \"choices\": [\"choice 1\", \"choice 2\", \"choice 3\", \"choice 4\"], \
                \"correct_answer\": \"choice 2\" \
              }} \
            ]",
            topic = topic,
            subtopic_list = subtopic_list,
            n = subtopics.len(),
        ),
        TriviaStyle::ShortAnswer => format!(
            "You are a professional trivia generator. For each of the following subtopics about '{topic}', \
            generate exactly one trivia question with a direct single short-answer (typically 1-2 words). \
            Subtopics: [{subtopic_list}]. \
            Your response must be a single raw valid JSON array with exactly {n} elements. \
            Do not wrap in markdown or any other tags. \
            The JSON schema is: \
            [ \
              {{ \
                \"question\": \"Question text?\", \
                \"correct_answer\": \"Answer\" \
              }} \
            ]",
            topic = topic,
            subtopic_list = subtopic_list,
            n = subtopics.len(),
        ),
    };

    let options = OllamaOptions {
        temperature: 0.6,
        ..Default::default()
    };

    info!(url=%url, model=%model, topic=%topic, "Generating questions from subtopics");

    let raw = ollama_call(client, url, model, prompt, options).await?;
    if LOG_LLM_OUTPUT {
        log_llm_output("questions", model, topic, &raw).await;
    }

    let mut questions: Vec<TriviaQuestion> = serde_json::from_str(&raw).map_err(|e| {
        warn!(response=%raw, error=%e, "Failed to parse trivia questions JSON");
        format!(
            "Invalid trivia JSON from model: {}. Raw response: {}",
            e, raw
        )
    })?;

    // Validate and normalise
    if style == TriviaStyle::MultipleChoice {
        for (i, q) in questions.iter_mut().enumerate() {
            match &q.choices {
                Some(choices) if choices.len() != 4 => {
                    return Err(format!(
                        "Question {} has invalid choices count (expected 4, got {})",
                        i + 1,
                        choices.len()
                    ));
                }
                Some(choices) if !choices.contains(&q.correct_answer) => {
                    return Err(format!(
                        "Question {} correct answer '{}' is not among the choices {:?}",
                        i + 1,
                        q.correct_answer,
                        choices
                    ));
                }
                None => {
                    return Err(format!(
                        "Question {} is missing choices for multiple-choice style",
                        i + 1
                    ));
                }
                _ => {}
            }
        }
    } else {
        for q in questions.iter_mut() {
            q.choices = None;
        }
    }

    info!(count=%questions.len(), "Successfully generated trivia questions");
    Ok(questions)
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub(crate) async fn load_trivia_questions(
    topic: &str,
    num_questions: u8,
    style: TriviaStyle,
    model: &str,
) -> Result<Vec<TriviaQuestion>, String> {
    let ollama_host =
        env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
    let url = format!("{}/api/generate", ollama_host);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    // Load the global subtopic cache, generate subtopics with the blocklist
    // injected, then persist the updated cache before returning.
    let mut cache = load_cache().await;

    let subtopics = generate_subtopics(&client, &url, model, topic, num_questions, &cache).await?;

    // Add the freshly generated subtopics to the cache.
    for subtopic in &subtopics {
        cache.insert(topic, subtopic.clone());
    }
    save_cache(&cache).await;

    let questions =
        generate_questions_from_subtopics(&client, &url, model, topic, &subtopics, style).await?;

    // Verification Pass //TODO

    Ok(questions)
}
