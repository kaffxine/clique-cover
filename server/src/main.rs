use axum::{
    response::Html,
    Router,
    routing::{get, post},
};
use std::{
    io,
    path::Path,
};
use tokio::{
    fs,
    net::TcpListener,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("heeey from axum :3");

    let app = Router::new()
        .route("/", get(serve_index));

    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn serve_index() -> Html<String> {
    let path = Path::new("/app/public/index.html");
    
    let content = match fs::read_to_string(path).await {
        Ok(contents) => contents,
        Err(_) => "<h1>404: file not found</h1>".to_string(),
    };

    Html(content)
}


