use axum::{Json, Router};
use axum::extract::{ConnectInfo, State};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
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

const ASSETS_DIR: &str = "/app/public";

#[derive(Clone)]
pub struct AppState {
    pub run_params: Arc<RwLock<RunParams>>,
    pub algo_list: Arc<RwLock<Vec<Algo>>>,
    pub tx_to_intern: broadcast::Sender<MsgToIntern>,
    pub tx_to_public: mpsc::Sender<MsgToPublic>,
    pub rx_to_public: Arc<Mutex<Option<mpsc::Receiver<MsgToPublic>>>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx_int, _) = broadcast::channel::<Msg>(SIZE);
        let (tx_ext, rx_ext) = mpsc::channel::<Msg>(SIZE);
        AppState {
            run_params: Arc::new(RwLock::new(RunParams {
                graph_gen_params: GraphGenParams::default(),
                algo_ids_selected: Vec::new(),
            })),
            algo_list: Arc::new(RwLock::new(Vec::new())),
            tx_to_intern = tx_int,
            tx_to_public = tx_ext,
            rx_to_public = Arc::new(Mutex::new(Some(rx_ext))),
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let app_state = AppState::new();

    let internal_router = get_internal_router(app_state.clone());
    let public_router = get_public_router(app_state.clone());

    let internal_listener = TcpListener::bind("127.0.0.1:3001").await?;
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
    let rx = if let Ok(rx) = state
        .rx_to_public
        .lock()
        .await
        .as_mut()
        .and_then(|opt| opt.take())
    { rx } 
    else {
        let msg = Msg::busy().to_json().unwrap();
        if socket
            .send(Message::Text(msg))
            .await
            .is_err()
        {
            break;
        }
    }

    while let Some(Ok(message)) = socket.recv().await {
        if let Message::Text(content) = message {
            let msg = if let Ok(msg) = Msg::unpack(content) {
                msg
            } else {
                println!("failed to parse json from the website");
                continue;
            };
            match msg.cmd.as_str() {
                "START" => {
                    
                },
                "ABORT" => {

                },
            }
        }
    }
}


async fn update_run_settings(
    State(app_state): State<AppState>,
    Json(received_settings): Json<RunParams>,
) -> &'static str {
    let mut lock = app_state.run_params.lock().unwrap();
    *lock = Some(received_settings);
    "settings updated successfully"
}

