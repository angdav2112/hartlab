//! HartLab session control plane (Phase 0/1 skeleton).
//!
//! `POST /v1/sessions` validates an ELF or example id. Docker launch lands
//! in the next slice; this binary is enough to exercise the accept list
//! and keep `cargo test --workspace` green.

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use hartlab_protocol::{validate_elf, ElfError};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

const MAX_CONCURRENT: usize = 4;

#[derive(Clone)]
struct App {
    live: Arc<Mutex<usize>>,
}

#[derive(Debug, Deserialize)]
struct CreateSession {
    target: Option<String>,
    example: Option<String>,
    /// Raw ELF bytes as base64. Prefer multipart later; this is for tests.
    elf_b64: Option<String>,
}

#[derive(Debug, Serialize)]
struct SessionCreated {
    id: String,
    target: String,
    example: Option<String>,
    ws_path: String,
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

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/v1/sessions", post(create_session))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(App {
            live: Arc::new(Mutex::new(0)),
        });

    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    tracing::info!("hartlab-control listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn healthz() -> impl IntoResponse {
    Json(serde_json::json!({ "ok": true, "service": "hartlab-control" }))
}

async fn create_session(
    State(app): State<App>,
    Json(body): Json<CreateSession>,
) -> impl IntoResponse {
    let target = body.target.unwrap_or_else(|| "polarfire".into());
    if target != "polarfire" {
        return err(StatusCode::BAD_REQUEST, "only target=polarfire is implemented");
    }

    if let Some(b64) = body.elf_b64.as_deref() {
        let Ok(bytes) = decode_b64(b64) else {
            return err(StatusCode::BAD_REQUEST, "elf_b64 is not valid base64");
        };
        if let Err(e) = validate_elf(&bytes) {
            return elf_err(e);
        }
    } else if body.example.is_none() {
        return err(StatusCode::BAD_REQUEST, "provide example or elf_b64");
    }

    let mut live = app.live.lock().await;
    if *live >= MAX_CONCURRENT {
        return err(StatusCode::SERVICE_UNAVAILABLE, "lab is full");
    }
    *live += 1;
    drop(live);

    let id = Uuid::new_v4().to_string();
    let created = SessionCreated {
        ws_path: format!("/v1/sessions/{id}/ws"),
        id,
        target,
        example: body.example,
    };
    (StatusCode::CREATED, Json(created)).into_response()
}

fn decode_b64(s: &str) -> Result<Vec<u8>, ()> {
    fn val(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let s = s.as_bytes();
    let mut out = Vec::with_capacity(s.len() / 4 * 3);
    let mut buf = 0u32;
    let mut n = 0;
    for &c in s {
        if c == b'=' || c.is_ascii_whitespace() {
            continue;
        }
        let Some(v) = val(c) else { return Err(()) };
        buf = (buf << 6) | u32::from(v);
        n += 6;
        if n >= 8 {
            n -= 8;
            out.push((buf >> n) as u8);
        }
    }
    Ok(out)
}

fn elf_err(e: ElfError) -> axum::response::Response {
    err(StatusCode::BAD_REQUEST, e.to_string())
}

fn err(status: StatusCode, message: impl Into<String>) -> axum::response::Response {
    (status, Json(ErrorBody { error: message.into() })).into_response()
}
