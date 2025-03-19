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

/// We'll chunk the user's input into smaller pieces (e.g., 4096 chars).
/// For each chunk, we spawn a task that calls `call_openai_tts`, then
/// save the returned MP3 to disk (e.g., "speech-chunk-1.mp3").
#[derive(Deserialize)]
pub struct UserInput {
    pub input: String,
}

// Utility to chunk text
fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut start = 0;
    while start < text.len() {
        let end = (start + chunk_size).min(text.len());
        result.push(text[start..end].to_string());
        start += chunk_size;
    }
    result
}

pub async fn speech(
    State(_state): State<AppState>,
    Json(payload): Json<UserInput>,
) -> (StatusCode, Json<serde_json::Value>) {
    // 1) Retrieve the API key
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(k) => k,
        Err(_) => {
            let err = json!({ "error": "Missing OPENAI_API_KEY" });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
        }
    };

    // 2) Split the user input into chunks
    let chunks = chunk_text(&payload.input, 4096);
    if chunks.is_empty() {
        let err = json!({ "error": "No text provided." });
        return (StatusCode::BAD_REQUEST, Json(err));
    }

    // 3) For each chunk, spawn a parallel task to call TTS
    let mut tasks = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        let api_key_cloned = api_key.clone();
        let chunk_cloned = chunk.clone();
        let voice = "onyx".to_string();
        let index = i + 1;

        tasks.push(task::spawn(async move {
            // Call TTS
            let tts_result = call_openai_tts(&api_key_cloned, &chunk_cloned, &voice).await;
            match tts_result {
                Ok(bytes) => {
                    // Save to "speech-chunk-{index}.mp3"
                    let filename = format!("speech-chunk-{}.mp3", index);
                    if let Err(e) = fs::write(&filename, &bytes) {
                        Err(format!("Failed to write {filename}: {e}"))
                    } else {
                        Ok(filename)
                    }
                }
                Err(msg) => Err(format!("Chunk {index} TTS error: {msg}")),
            }
        }));
    }

    // 4) Wait for all tasks to finish in parallel
    let results = join_all(tasks).await;

    let mut saved_files = Vec::new();
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(filename)) => {
                // That chunk was saved successfully
                saved_files.push(filename);
            }
            Ok(Err(e)) => {
                // TTS or file writing error
                let err = json!({ "error": format!("Task #{} error: {e}", i+1) });
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
            }
            Err(join_err) => {
                // The task panicked or was cancelled
                let err = json!({ "error": format!("Join error on task #{}: {join_err}", i+1) });
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(err));
            }
        }
    }

    // 5) Return a JSON response listing the saved files
    let response = json!({
        "message": "All chunks processed in parallel.",
        "files": saved_files,
    });
    (StatusCode::OK, Json(response))
}
