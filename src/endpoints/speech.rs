use crate::services::tts_service::call_openai_tts;
use crate::state::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use futures::future::join_all;
use serde::Deserialize;
use serde_json::json;
use std::{env, fs};
use tokio::task;

// 1) Bring in the function you wrote above
use crate::utils::chunk_text_unicode::chunk_text_unicode;

#[derive(Deserialize)]
pub struct UserInput {
    pub input: String,
}

pub async fn speech(
    State(_state): State<AppState>,
    Json(payload): Json<UserInput>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 2) Check OPENAI_API_KEY
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            let err = json!({ "error": "Missing OPENAI_API_KEY" });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
        }
    };

    // 3) Safely chunk the input at Unicode boundaries, up to 4096 graphemes
    let chunks = chunk_text_unicode(&payload.input, 4096);
    if chunks.is_empty() {
        let err = json!({ "error": "No text provided." });
        return (StatusCode::BAD_REQUEST, Json(err));
    }

    // 4) For each chunk, spawn a parallel task
    let mut tasks = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let api_key_cloned = api_key.clone();
        let chunk_cloned = chunk.clone();
        let voice = "onyx".to_string();
        let index = i + 1;

        tasks.push(task::spawn(async move {
            let tts_result = call_openai_tts(&api_key_cloned, &chunk_cloned, &voice).await;
            match tts_result {
                Ok(bytes) => {
                    let filename = format!("speech-chunk-{index}.mp3");
                    match fs::write(&filename, &bytes) {
                        Ok(_) => Ok(filename),
                        Err(e) => Err(format!("Failed to write {filename}: {e}")),
                    }
                }
                Err(msg) => Err(format!("Chunk {index} TTS error: {msg}")),
            }
        }));
    }

    // 5) Wait for all tasks
    let results = join_all(tasks).await;

    let mut saved_files = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(filename)) => {
                saved_files.push(filename);
            }
            Ok(Err(e)) => {
                let err = json!({ "error": format!("Task #{} error: {e}", i+1) });
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
            }
            Err(join_err) => {
                let err = json!({ "error": format!("Join error on task #{}: {join_err}", i+1) });
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
            }
        }
    }

    // 6) Return success
    let response = json!({
        "message": "All chunks processed in parallel.",
        "files": saved_files,
    });
    (StatusCode::OK, Json(response))
}
