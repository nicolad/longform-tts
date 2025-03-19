use crate::state::AppState; // from your crate::state
use axum::{extract::State, http::StatusCode, Json};
use reqwest::Client;
use serde::Serialize;
use serde_json::json;
use std::env;

#[derive(Serialize)]
struct TtsRequest {
    model: String,
    input: String,
    voice: String,
}

pub async fn speech(State(_state): State<AppState>) -> (StatusCode, Json<serde_json::Value>) {
    // 1) Read your API key from an environment variable
    let api_key =
        env::var("OPENAI_API_KEY").expect("Please set the OPENAI_API_KEY environment variable");

    // 2) Create an HTTP client
    let client = Client::new();

    // 3) Build the TTS request body
    let body = TtsRequest {
        model: "tts-1".to_string(),
        input: "Hello world, this is a test.".to_string(),
        voice: "onyx".to_string(),
    };

    // 4) POST to the speech endpoint
    let resp = match client
        .post("https://api.openai.com/v1/audio/speech")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            // If we failed to send the request at all, return a 500
            let error_body = json!({ "error": format!("Request error: {e}") });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_body));
        }
    };

    // 5) Check if it succeeded
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        eprintln!("TTS request failed: {status} - {text}");

        // Return a 400 or 500 with an error message
        let error_body = json!({
            "error": format!("TTS request failed: {status} - {text}"),
        });
        return (StatusCode::BAD_REQUEST, Json(error_body));
    }

    // 6) Save the returned MP3 bytes to "speech.mp3"
    let audio_bytes = match resp.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            let error_body = json!({ "error": format!("Unable to get response body: {e}") });
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_body));
        }
    };

    if let Err(e) = std::fs::write("speech.mp3", &audio_bytes) {
        let error_body = json!({ "error": format!("Failed to write file: {e}") });
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_body));
    }

    println!("Saved output to speech.mp3");

    // 7) Return the JSON response
    let response = json!({
        "message": "Audio saved successfully",
        "file_path": "speech.mp3"
    });
    (StatusCode::OK, Json(response))
}
