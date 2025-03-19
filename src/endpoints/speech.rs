// speech.rs
use crate::services::tts_service::call_openai_tts; // <-- import from your new file
use crate::state::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Deserialize)]
pub struct UserInput {
    pub input: String,
}

pub async fn speech(
    State(_state): State<AppState>,
    Json(payload): Json<UserInput>,
) -> (StatusCode, Json<serde_json::Value>) {
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            let err = json!({ "error": "Missing OPENAI_API_KEY" });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
        }
    };

    // Call your TTS service
    let audio_bytes = match call_openai_tts(&api_key, &payload.input, "onyx").await {
        Ok(bytes) => bytes,
        Err(msg) => {
            let err = json!({ "error": msg });
            return (StatusCode::BAD_REQUEST, Json(err));
        }
    };

    if let Err(e) = std::fs::write("speech.mp3", &audio_bytes) {
        let err = json!({ "error": format!("Failed to write file: {e}") });
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
    }

    let response = json!({
        "message": "Audio saved successfully",
        "file_path": "speech.mp3"
    });
    (StatusCode::OK, Json(response))
}
