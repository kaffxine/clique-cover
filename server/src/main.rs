use axum::{
    extract::State,
    Json,
    response::Html,
    Router,
    routing::{get, get_service, post},
};
use std::{
    io,
    path::Path,
    sync::{Arc, Mutex},
};
use tokio::{
    fs,
    net::TcpListener,
};
use tower_http::services::ServeDir;

mod settings;
use settings::{Settings};

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("heeey from axum :3");

    let app_state = AppState {
        settings: Arc::new(Mutex::new(None)),
    };

    let app = Router::new()
        .route("/settings", post(update_settings))
        .fallback_service(ServeDir::new("/app/public"))
        .with_state(app_state);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Clone)]
struct AppState {
    settings: Arc<Mutex<Option<Settings>>>
}

async fn update_settings(
    State(state): State<AppState>,
    Json(received_settings): Json<Settings>,
) -> &'static str {
    let mut lock = state.settings.lock().unwrap();
    *lock = Some(received_settings);
    "settings updated successfully"
}
    

// async fn serve_index() -> Html<String> {
//     let path = Path::new("/app/public/index.html");
//     
//     let content = match fs::read_to_string(path).await {
//         Ok(contents) => contents,
//         Err(_) => "<h1>404: file not found</h1>".to_string(),
//     };
// 
//     Html(content)
// }


