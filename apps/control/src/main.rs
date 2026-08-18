//! HartLab control-plane **stub**.
//!
//! Validates example ids / uploaded ELFs. Does **not** start Docker or a
//! WebSocket. Phase 0 exit is live Renode, not more Axum surface.

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use hartlab_protocol::{validate_elf, ElfError};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

#[derive(Debug, Deserialize)]
struct CreateSession {
    target: Option<String>,
    example: Option<String>,
    elf_b64: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionPreview {
    target: String,
    example: Option<String>,
    elf_ok: bool,
    stub: bool,
    note: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hartlab_control=info,tower_http=info".into()),
        )
        .init();

    let bind = std::env::var("HARTLAB_BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let addr: SocketAddr = bind.parse().expect("HARTLAB_BIND must be host:port");
    if std::env::var("HARTLAB_MAX_CONCURRENT").is_ok() {
        tracing::info!("HARTLAB_MAX_CONCURRENT is recorded for Phase 1; this stub starts no sessions");
    }

    let cors = CorsLayer::new().allow_origin(AllowOrigin::predicate(|origin, _| {
        origin
            .to_str()
            .map(|s| s.starts_with("http://127.0.0.1") || s.starts_with("http://localhost"))
            .unwrap_or(false)
    }));

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/sessions", post(preview_session))
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    tracing::info!("hartlab-control stub listening on {addr} (no docker, no ws)");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({
        "ok": true,
        "service": "hartlab-control",
        "stub": true,
    }))
}

async fn preview_session(Json(body): Json<CreateSession>) -> impl IntoResponse {
    let target = body.target.unwrap_or_else(|| "polarfire".into());
    if target != "polarfire" {
        return err(StatusCode::BAD_REQUEST, "only target=polarfire is implemented");
    }

    let mut elf_ok = false;
    if let Some(b64) = body.elf_b64.as_deref() {
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) else {
            return err(StatusCode::BAD_REQUEST, "elf_b64 is not valid base64");
        };
        if let Err(e) = validate_elf(&bytes) {
            return elf_err(e);
        }
        elf_ok = true;
    } else if body.example.is_none() {
        return err(StatusCode::BAD_REQUEST, "provide example or elf_b64");
    }

    let preview = SessionPreview {
        target,
        example: body.example,
        elf_ok,
        stub: true,
        note: "placeholder: no container, no websocket. finish the Renode fidelity spike first.",
    };
    (StatusCode::OK, Json(preview)).into_response()
}

fn elf_err(e: ElfError) -> axum::response::Response {
    err(StatusCode::BAD_REQUEST, e.to_string())
}

fn err(status: StatusCode, message: impl Into<String>) -> axum::response::Response {
    (status, Json(ErrorBody { error: message.into() })).into_response()
}
