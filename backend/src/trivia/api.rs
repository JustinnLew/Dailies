use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, warn};

use crate::state::trivia::{TriviaQuestion, TriviaStyle};

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
        .map_err(|e| format!("Ollama connection error: {}. Make sure Ollama is running at {} and the model is pulled.", e, url))?;

    if !response.status().is_success() {
        let status = response.status();
        let err_text = response.text().await.unwrap_or_default();
        warn!(status=?status, err=%err_text, "Ollama returned error status");
        return Err(format!("Ollama server returned error {}: {}", status, err_text));
    }

    let gen_response = response
        .json::<OllamaGenerateResponse>()
        .await
        .map_err(|e| format!("Failed to parse Ollama envelope: {}", e))?;

    Ok(gen_response.response)
}

/// Step 1 — generate n well-known, distinct subtopics for the given topic.
async fn generate_subtopics(
    client: &reqwest::Client,
    url: &str,
    model: &str,
    topic: &str,
    num_questions: u8,
) -> Result<Vec<String>, String> {
    let prompt = format!(
        "Generate exactly {num_questions} x 3 trivia subtopics about '{topic}'. \
        Each subtopic must be: \
        (1) well-known enough that a general audience would recognise it, \
        (2) clearly distinct from the others with no overlapping themes, \
        (3) specific enough to produce one focused trivia question. \
        Then pick a random {num_questions} of these subtopics to return.
        Your response must be a single raw JSON array of strings. \
        Do not wrap in markdown or any other tags. \
        Example: [\"subtopic 1\", \"subtopic 2\"]"
    );

    let options = OllamaOptions {
        temperature: 1.0,
        top_k: 80,
        top_p: 0.95,
        ..Default::default()
    };

    info!(url=%url, model=%model, topic=%topic, "Generating subtopics");

    let raw = ollama_call(client, url, model, prompt, options).await?;

    let subtopics: Vec<String> = serde_json::from_str(&raw).map_err(|e| {
        warn!(response=%raw, error=%e, "Failed to parse subtopics JSON");
        format!("Invalid subtopics JSON from model: {}. Raw response: {}", e, raw)
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

    let mut questions: Vec<TriviaQuestion> = serde_json::from_str(&raw).map_err(|e| {
        warn!(response=%raw, error=%e, "Failed to parse trivia questions JSON");
        format!("Invalid trivia JSON from model: {}. Raw response: {}", e, raw)
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

    let subtopics = generate_subtopics(&client, &url, model, topic, num_questions).await?;

    let questions =
        generate_questions_from_subtopics(&client, &url, model, topic, &subtopics, style).await?;

    // Verification Pass //TODO

    Ok(questions)
}