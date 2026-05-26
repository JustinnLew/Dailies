use std::{
    collections::HashMap,
    env,
    sync::{Arc, Mutex},
};

use axum::extract::ws::{Message, WebSocket};
use futures_util::{
    SinkExt, StreamExt,
    stream::{SplitSink, SplitStream},
};
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, broadcast};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    connections::ConnectionManager,
    state::{LobbyServerEvent, LobbyState, LobbyStatus, LobbyUserEvent},
    trivia::api,
};

/// ===============================================
/// Enums
/// ===============================================
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum TriviaStyle {
    MultipleChoice,
    ShortAnswer,
}

/// ===============================================
/// Settings
/// ===============================================
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TriviaSettings {
    pub topic: String,
    pub num_questions: u8,
    pub round_length_seconds: u8,
    pub style: TriviaStyle,
}

impl TriviaSettings {
    pub fn new() -> Self {
        Self {
            topic: "General Knowledge".to_string(),
            num_questions: 10,
            round_length_seconds: 20,
            style: TriviaStyle::MultipleChoice,
        }
    }
}

/// ===============================================
/// Questions
/// ===============================================
#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct TriviaQuestion {
    pub question: String,
    pub choices: Option<Vec<String>>,
    pub correct_answer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TriviaQuestionPublic {
    pub question: String,
    pub choices: Option<Vec<String>>,
}

/// ===============================================
/// State
/// ===============================================
pub(crate) struct TriviaState {
    pub scores: HashMap<Uuid, u32>,
    pub questions: Vec<TriviaQuestion>,
    pub question_index: usize,
    // current_round_guesses: player_id -> guess_content
    pub current_round_guesses: HashMap<Uuid, String>,
}

impl TriviaState {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            questions: Vec::new(),
            question_index: 0,
            current_round_guesses: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.scores.iter_mut().for_each(|(_, score)| *score = 0);
        self.questions.clear();
        self.question_index = 0;
        self.current_round_guesses.clear();
    }
}

/// ===============================================
/// Main Game Parent Struct
/// ===============================================
pub(crate) struct TriviaGame {
    pub lobby_state: Mutex<LobbyState>,
    pub broadcast: broadcast::Sender<TriviaServerEvent>,
    pub settings: Mutex<TriviaSettings>,
    pub state: Mutex<TriviaState>,
    pub lobby_code: String,
    pub round_notify: Mutex<Arc<Notify>>,
}

impl TriviaGame {
    pub async fn await_join_req(
        receiver: &mut SplitStream<WebSocket>,
        sender: &mut SplitSink<WebSocket, Message>,
    ) -> Result<(String, String), ()> {
        let join_req = match receiver.next().await {
            Some(Ok(Message::Text(m))) => m,
            _ => return Err(()),
        };
        let event = match serde_json::from_str::<TriviaClientEvent>(&join_req) {
            Ok(e) => e,
            Err(_) => {
                info!("JOIN ERROR");
                let _ = sender
                    .send(Message::Text(
                        serde_json::to_string(&TriviaServerEvent::LobbyEvent(
                            LobbyServerEvent::JoinError {
                                message: "Failed to serialize Initial Request".to_string(),
                            },
                        ))
                        .unwrap()
                        .into(),
                    ))
                    .await;
                return Err(());
            }
        };
        let (lobby_code, player_username) = match event {
            TriviaClientEvent::LobbyEvent(LobbyUserEvent::Join {
                lobby_code,
                username,
            }) => (lobby_code, username),
            _ => {
                info!("JOIN ERROR");
                let _ = sender
                    .send(Message::Text(
                        serde_json::to_string(&TriviaServerEvent::LobbyEvent(
                            LobbyServerEvent::JoinError {
                                message: "Expected Join Event".to_string(),
                            },
                        ))
                        .unwrap()
                        .into(),
                    ))
                    .await;
                return Err(());
            }
        };
        Ok((lobby_code, player_username))
    }

    pub fn handle_lobby_event(self: &Arc<Self>, player_id: Uuid, event: LobbyUserEvent) {
        match event {
            LobbyUserEvent::Ready => {
                self.player_ready(&player_id);
                let _ = self.broadcast.send(TriviaServerEvent::LobbyEvent(
                    LobbyServerEvent::PlayerReady {
                        player_id: player_id.clone(),
                    },
                ));

                if self.all_ready() && self.get_lobby_status() == LobbyStatus::Waiting {
                    let _ = self
                        .broadcast
                        .send(TriviaServerEvent::GameEvent(TriviaGameEvent::AllReady));
                    self.update_lobby_status(LobbyStatus::Loading);

                    let l = Arc::clone(self);
                    tokio::spawn(async move {
                        let settings = l.get_settings();
                        let ollama_model =
                            env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
                        let res = api::load_trivia_questions(
                            &settings.topic,
                            settings.num_questions,
                            settings.style,
                            &ollama_model,
                        )
                        .await;

                        match res {
                            Ok(questions) => {
                                {
                                    let mut state = l.state.lock().unwrap();
                                    state.questions = questions;
                                }
                                l.update_lobby_status(LobbyStatus::Playing);
                                TriviaGame::run_game(l).await;
                            }
                            Err(msg) => {
                                warn!("Trivia generation error: {}", msg);
                                l.update_lobby_status(LobbyStatus::Waiting);
                                l.player_unready(&player_id);
                                let _ = l.broadcast.send(TriviaServerEvent::LobbyEvent(
                                    LobbyServerEvent::PlayerUnready { player_id },
                                ));
                                let _ = l.broadcast.send(TriviaServerEvent::GameEvent(
                                    TriviaGameEvent::LoadingError { message: msg },
                                ));
                            }
                        }
                    });
                }
            }
            LobbyUserEvent::Unready => {
                self.player_unready(&player_id);
                let _ = self.broadcast.send(TriviaServerEvent::LobbyEvent(
                    LobbyServerEvent::PlayerUnready { player_id },
                ));
            }
            _ => {}
        }
    }

    pub fn handle_game_event(&self, player_id: Uuid, event: TriviaUserGameEvent) {
        match event {
            TriviaUserGameEvent::UpdateGameSettings { settings } => {
                if self.get_lobby_status() == LobbyStatus::Waiting {
                    self.update_game_settings(settings.clone());
                    let _ = self.broadcast.send(TriviaServerEvent::GameEvent(
                        TriviaGameEvent::GameSettingsUpdated { settings },
                    ));
                }
            }
            TriviaUserGameEvent::Guess { content } => {
                if self.get_lobby_status() != LobbyStatus::Playing {
                    return;
                }

                let style = self.get_settings().style;
                match style {
                    TriviaStyle::ShortAnswer => {
                        if self.is_correct_short_answer(&content) {
                            self.increment_score(&player_id);
                            let correct_val = {
                                let state = self.state.lock().unwrap();
                                state.questions[state.question_index - 1]
                                    .correct_answer
                                    .clone()
                            };
                            let username = {
                                let lobby = self.lobby_state.lock().unwrap();
                                lobby
                                    .players
                                    .get(&player_id)
                                    .map(|p| p.0.clone())
                                    .unwrap_or_default()
                            };
                            let _ = self.broadcast.send(TriviaServerEvent::GameEvent(
                                TriviaGameEvent::CorrectGuess {
                                    player_id,
                                    msg: format!(
                                        "{} guessed correctly! The correct answer was '{}'.",
                                        username, correct_val
                                    ),
                                },
                            ));
                            self.round_notify.lock().unwrap().notify_one();
                        } else {
                            let username = {
                                let lobby = self.lobby_state.lock().unwrap();
                                lobby
                                    .players
                                    .get(&player_id)
                                    .map(|p| p.0.clone())
                                    .unwrap_or_default()
                            };
                            let _ = self.broadcast.send(TriviaServerEvent::GameEvent(
                                TriviaGameEvent::PlayerGuess { username, content },
                            ));
                        }
                    }
                    TriviaStyle::MultipleChoice => {
                        if self.record_guess(player_id.clone(), content).is_ok() {
                            let username = {
                                let lobby = self.lobby_state.lock().unwrap();
                                lobby
                                    .players
                                    .get(&player_id)
                                    .map(|p| p.0.clone())
                                    .unwrap_or_default()
                            };
                            let _ = self.broadcast.send(TriviaServerEvent::GameEvent(
                                TriviaGameEvent::PlayerGuess {
                                    username,
                                    content: "Submitted answer".to_string(),
                                },
                            ));

                            if self.all_players_guessed() {
                                self.round_notify.lock().unwrap().notify_one();
                            }
                        }
                    }
                }
            }
        }
    }

    pub async fn run_game(game: Arc<Self>) {
        info!("Starting Trivia game loop");
        let _ = game
            .broadcast
            .send(TriviaServerEvent::GameEvent(TriviaGameEvent::GameStart));
        sleep(Duration::from_secs(3)).await;

        let settings = game.get_settings();

        for _ in 0..settings.num_questions {
            if game.no_connections() {
                info!("Game empty, terminating loop");
                return;
            }

            // Initialize round
            game.begin_round();

            let question = match game.get_next_question() {
                Some(q) => q,
                None => {
                    info!("No questions left, ending game");
                    break;
                }
            };

            let round_notify = Arc::new(tokio::sync::Notify::new());
            *game.round_notify.lock().unwrap() = Arc::clone(&round_notify);

            info!(question=%question.question, "ROUND START");
            let _ =
                game.broadcast
                    .send(TriviaServerEvent::GameEvent(TriviaGameEvent::RoundStart {
                        question,
                    }));

            // Await round length or early notification
            tokio::select! {
                _ = sleep(Duration::from_secs(settings.round_length_seconds as u64)) => {
                    info!("Round ended (timer elapsed)");
                }
                _ = round_notify.notified() => {
                    info!("Round ended early (correct guess or all players answered)");
                }
            }

            if game.no_connections() {
                info!("Game empty during round, terminating loop");
                return;
            }

            // For Multiple Choice, score active guesses at the end of the round
            if settings.style == TriviaStyle::MultipleChoice {
                game.score_multiple_choice_round();
            }

            let correct_answer = {
                let state = game.state.lock().unwrap();
                state.questions[state.question_index - 1]
                    .correct_answer
                    .clone()
            };

            info!("ROUND END");
            let _ = game
                .broadcast
                .send(TriviaServerEvent::GameEvent(TriviaGameEvent::RoundEnd {
                    correct_answer,
                    leaderboard: game.get_leaderboard(),
                }));

            // Delay between rounds hardcoded to 3 seconds as requested
            sleep(Duration::from_secs(3)).await;
        }

        info!("TRIVIA GAME END");
        let _ = game
            .broadcast
            .send(TriviaServerEvent::GameEvent(TriviaGameEvent::GameEnd));
        game.reset();
    }

    pub fn reset(&self) {
        self.lobby_state.lock().unwrap().reset();
        self.state.lock().unwrap().reset();
    }

    pub fn get_new_player_id(&self) -> Uuid {
        self.lobby_state.lock().unwrap().get_new_player_id()
    }

    pub fn player_join(&self, player_id: Uuid, player_username: String) -> Result<(), &str> {
        let mut lobby = self.lobby_state.lock().unwrap();
        let mut state = self.state.lock().unwrap();
        if lobby.status != LobbyStatus::Waiting {
            return Err("Cannot join game in progress");
        }
        lobby.player_join(player_id.clone(), player_username);
        state.scores.insert(player_id.clone(), 0);
        Ok(())
    }

    pub fn get_players(&self) -> Vec<(Uuid, String, bool)> {
        self.lobby_state.lock().unwrap().get_players()
    }

    pub fn player_ready(&self, user_id: &Uuid) {
        self.lobby_state.lock().unwrap().player_ready(user_id);
    }

    pub fn player_unready(&self, user_id: &Uuid) {
        self.lobby_state.lock().unwrap().player_unready(user_id);
    }

    pub fn all_ready(&self) -> bool {
        self.lobby_state.lock().unwrap().all_ready()
    }

    pub fn get_lobby_status(&self) -> LobbyStatus {
        self.lobby_state.lock().unwrap().status.clone()
    }

    pub fn update_lobby_status(&self, status: LobbyStatus) {
        let mut lobby = self.lobby_state.lock().unwrap();
        lobby.status = status;
    }

    pub fn get_settings(&self) -> TriviaSettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn get_leaderboard(&self) -> HashMap<Uuid, u32> {
        self.state.lock().unwrap().scores.clone()
    }

    pub fn get_next_question(&self) -> Option<TriviaQuestionPublic> {
        let mut state = self.state.lock().unwrap();
        state.question_index += 1;
        if state.questions.is_empty()
            || state.question_index == 0
            || state.question_index - 1 >= state.questions.len()
        {
            return None;
        }
        let q = &state.questions[state.question_index - 1];
        Some(TriviaQuestionPublic {
            question: q.question.clone(),
            choices: q.choices.clone(),
        })
    }

    pub fn update_game_settings(&self, settings: TriviaSettings) {
        let mut game_settings = self.settings.lock().unwrap();
        *game_settings = settings;
    }

    pub fn is_correct_short_answer(&self, guess: &str) -> bool {
        let state = self.state.lock().unwrap();
        if state.questions.is_empty()
            || state.question_index == 0
            || state.question_index - 1 >= state.questions.len()
        {
            return false;
        }
        let correct_answer = &state.questions[state.question_index - 1].correct_answer;
        let correct = correct_answer.trim().to_lowercase();
        let guess = guess.trim().to_lowercase();
        correct == guess || strsim::damerau_levenshtein(&correct, &guess) <= 1
    }

    pub fn record_guess(&self, player_id: Uuid, guess: String) -> Result<(), &str> {
        let mut state = self.state.lock().unwrap();
        if state.current_round_guesses.contains_key(&player_id) {
            return Err("You have already guessed this round");
        }
        state.current_round_guesses.insert(player_id, guess);
        Ok(())
    }

    pub fn all_players_guessed(&self) -> bool {
        let state = self.state.lock().unwrap();
        let players = self.lobby_state.lock().unwrap().players.len();
        players > 0 && state.current_round_guesses.len() >= players
    }

    pub fn score_multiple_choice_round(&self) {
        let mut state = self.state.lock().unwrap();
        if state.questions.is_empty()
            || state.question_index == 0
            || state.question_index - 1 >= state.questions.len()
        {
            return;
        }
        let correct_answer = state.questions[state.question_index - 1]
            .correct_answer
            .trim()
            .to_lowercase();
        let guesses = state.current_round_guesses.clone();
        for (player_id, guess) in guesses {
            if guess.trim().to_lowercase() == correct_answer {
                if let Some(score) = state.scores.get_mut(&player_id) {
                    *score += 1;
                }
            }
        }
    }

    pub fn increment_score(&self, player_id: &Uuid) {
        let mut state = self.state.lock().unwrap();
        if let Some(score) = state.scores.get_mut(player_id) {
            *score += 1;
        }
    }

    pub fn begin_round(&self) {
        let mut state = self.state.lock().unwrap();
        state.current_round_guesses.clear();
    }
}

impl ConnectionManager for TriviaGame {
    fn connection_drop(&self, player_id: Uuid) {
        self.lobby_state.lock().unwrap().player_leave(&player_id);
        self.state.lock().unwrap().scores.remove(&player_id);
        let _ = self.broadcast.send(TriviaServerEvent::LobbyEvent(
            LobbyServerEvent::PlayerLeave { player_id },
        ));
    }

    fn no_connections(&self) -> bool {
        self.lobby_state.lock().unwrap().empty()
    }

    fn lobby_code(&self) -> String {
        self.lobby_code.clone()
    }
}

/// ===============================================
/// Events
/// ===============================================
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "data")]
pub(crate) enum TriviaServerEvent {
    LobbyEvent(LobbyServerEvent),
    GameEvent(TriviaGameEvent),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event", content = "data")]
pub(crate) enum TriviaGameEvent {
    SyncState {
        players: Vec<(Uuid, String, bool)>,
        settings: TriviaSettings,
        leaderboard: HashMap<Uuid, u32>,
        status: LobbyStatus,
    },
    AllReady,
    GameStart,
    GameSettingsUpdated {
        settings: TriviaSettings,
    },
    RoundStart {
        question: TriviaQuestionPublic,
    },
    RoundEnd {
        correct_answer: String,
        leaderboard: HashMap<Uuid, u32>,
    },
    GameEnd,
    PlayerGuess {
        username: String,
        content: String,
    },
    CorrectGuess {
        player_id: Uuid,
        msg: String,
    },
    LoadingError {
        message: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "event")]
pub(crate) enum TriviaUserGameEvent {
    UpdateGameSettings { settings: TriviaSettings },
    Guess { content: String },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
pub(crate) enum TriviaClientEvent {
    LobbyEvent(LobbyUserEvent),
    GameEvent(TriviaUserGameEvent),
}
