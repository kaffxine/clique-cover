use axum::{Json, Router};
use axum::extract::{ConnectInfo, State};
use axum::extract::ws::{WebSocket, WebSocketUpgrade};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, get_service, post};
use std::io;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{self, AtomicBool};
use tokio::fs;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod types;
use types::{AppState, Algo, RunParams, GraphGenParams, MsgToIntern, MsgToExtern};

const ASSETS_DIR: &str = "/app/public";

#[tokio::main]
async fn main() -> io::Result<()> {
    let app_state = AppState::new();

    let internal_router = get_internal_router(app_state.clone());
    let public_router = get_public_router(app_state.clone());

    let internal_listener = TcpListener::bind("0.0.0.0:3001").await?;
    let public_listener = TcpListener::bind("0.0.0.0:3000").await?;

    tokio::join!(
        axum::serve(internal_listener, internal_router.into_make_service()).await?,
        axum::serve(public_listener, public_router.into_make_service()).await?,
    );

    Ok(())
}

fn get_internal_router(app_state: AppState) -> Router {
    Router::new()
        .route("/ws", any(internal_ws_handler))
        .with_state(app_state)
}

fn get_public_router(app_state: AppState) -> Router {
    Router::new()
        .fallback_service(ServeDir::new(ASSETS_DIR))
        i
        .route("/ws", any(public_ws_handler))
        .with_state(app_state)
}

async fn internal_ws_handler(
    ws: WebSocketUpgrade,
    state: AppState,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_internal_ws(socket, state))
}

async fn public_ws_handler(
    ws: WebSocketUpgrade,
    state: AppState,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_internal_ws(socket, state))
}

async fn handle_internal_ws(mut socket: WebSocket state: AppState) {
    
}

async fn handle_public_ws(mut socket: WebSocket state: AppState) {

}


async fn update_run_settings(
    State(app_state): State<AppState>,
    Json(received_settings): Json<RunParams>,
) -> &'static str {
    let mut lock = app_state.run_params.lock().unwrap();
    *lock = Some(received_settings);
    "settings updated successfully"
}
