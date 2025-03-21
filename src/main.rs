use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION},
        Method,
    },
    routing::{get, post},
    Router,
};

pub mod endpoints;
pub mod services;
pub mod state;
pub mod utils;

use crate::endpoints::speech::speech;
use shuttle_openai::async_openai::{config::OpenAIConfig, Client};
use shuttle_runtime::DeploymentMetadata;
use shuttle_runtime::SecretStore;
use state::AppState;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres] conn: String,
    #[shuttle_openai::OpenAI(api_key = "{secrets.OPENAI_API_KEY}")] openai: Client<OpenAIConfig>,
    #[shuttle_runtime::Metadata] metadata: DeploymentMetadata,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_axum::ShuttleAxum {
    let state = AppState::new(conn, openai)
        .await
        .map_err(|e| format!("Could not create application state: {e}"))
        .unwrap();

    state.seed().await;

    let openai_api_key = secrets.get("OPENAI_API_KEY").unwrap();
    std::env::set_var("OPENAI_API_KEY", &openai_api_key);

    let origin = if metadata.env == shuttle_runtime::Environment::Deployment {
        format!("{}.shuttle.app", metadata.project_name)
    } else {
        "127.0.0.1:8000".to_string()
    };

    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_origin(vec![origin.parse().unwrap()])
        .allow_headers(vec![AUTHORIZATION, ACCEPT])
        .allow_methods(vec![Method::GET, Method::POST]);

    let router = Router::new()
        .route("/api/health", get(endpoints::health_check))
        .route("/api/auth/register", post(endpoints::auth::register))
        .route("/api/auth/login", post(endpoints::auth::login))
        .route(
            "/api/chat/conversations",
            get(endpoints::openai::get_conversation_list),
        )
        .route("/api/speech", post(speech))
        .route(
            "/api/chat/conversations/:id",
            get(endpoints::openai::fetch_conversation_messages)
                .post(endpoints::openai::send_message),
        )
        .route("/api/chat/create", post(endpoints::openai::create_chat))
        .layer(cors)
        .nest_service(
            "/",
            ServeDir::new("dist").not_found_service(ServeFile::new("dist/index.html")),
        )
        .with_state(state);

    Ok(router.into())
}
