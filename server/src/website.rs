use futures::{SinkExt, StreamExt};
use futures_util::TryStreamExt;
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody};
use hyper::{Request, Response, StatusCode, Server};
use hyper::body::{Body, Incoming, Frame};
use hyper::header;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade;
use hyper_util::rt::tokio::TokioIo;
use std::convert::Infallible;
use std::error::Error;
use std::io::Error as IoError;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::WebSocketStream;
use tokio_util::io::ReaderStream;

use shared::{Result, MyMsg};

use crate::types::{SharedState, Database};

async fn handle_ws(
    mut ws: WebSocketStream<upgrade::Upgraded>,
    rx_at_website: broadcast::Receiver<MyMsg>,
    tx_to_algonet: broadcast::Sender<MyMsg>,
    tx_to_grafnet: mpsc::Sender<MyMsg>,
    shared_state: Arc<SharedState>,
) {
}

async fn serve_file(path: &Path) -> Result<Response<Body>> {
    if path.components().any(|c| matches!(std::path::Component::ParentDir)) {
        return Ok(bad_request());
    }

    let file = File::open(path).await;
    if file.is_err() {
        eprintln!("unable to open file");
        return Ok(not_found());
    }
    let file = file.unwrap();

    let reader_stream = ReaderStream::new(file);
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = stream_body.boxed();
    
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .as_ref()
        .to_string();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_type)
        .body(boxed_body)
        .unwrap())
}

async fn handle_request(
    req: Request<Incoming>,
    public_dir: Arc<PathBuf>,
    rx_at_website: broadcast::Receiver<MyMsg>,
    tx_to_algonet: broadcast::Sender<MyMsg>,
    tx_to_grafnet: mpsc::Sender<MyMsg>,
    shared_state: Arc<SharedState>,
) -> Result<Response<Body>> {
    let path = req.uri().path();
    match path {
        "/" => {
            match serve_file(Path::new(public_dir.join("index.html"))).await {
                Ok(res) => Ok(res),
                Err(_) => Ok(internal_server_error()),
            }
        },
        "/ws" => {
            if let None = req.headers().get(header::UPGRADE) {
                eprintln!("no upgrade header in request");
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("bad request".into())
                    .unwrap();
            }
            let response = match upgrade::on(req).await {
                Ok(upgraded) => {
                    tokio::spawn(async move {
                        let stream = WebSocketStream::from_raw_socket(
                            upgraded,
                            tungstenite::protocol::Role::Server,
                            None,
                        ).await;
                        handle_ws(
                            stream,
                            rx_at_website,
                            tx_to_algonet,
                            tx_to_grafnet,
                            shared_state,
                        ).await;
                    });

                    Response::builder()
                        .status(StatusCode::SWITCHING_PROTOCOLS)
                        .header("Upgrade", "websocket")
                        .header("Connection", "Upgrade")
                        .header("Sec-WebSocket-Accept", "")
                        .body(Body::empty())
                        .unwrap()
                },
                Err(e) => {
                    eprintln!("upgrade error: {}", e);
                    Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body("bad request".into())
                        .unwrap(),
                }
        },
        _ => {
            if path.contains("..") {
                return Ok(bad_request());
            }
            let trimmed_path = path.trim_start_matches('/');
            match serve_file(Path::new(public_dir.join(trimmed_path))).await {
                Ok(res) => Ok(res),
                Err(_) => Ok(internal_server_error()),
            }
        },
    }
}

pub async fn handle_website(
    socket_addr: SocketAddr,
    public_dir: PathBuf,
    rx_at_website: broadcast::Receiver<MyMsg>,
    tx_to_algonet: broadcast::Sender<MyMsg>,
    tx_to_grafnet: mpsc::Sender<MyMsg>,
    shared_state: Arc<SharedState>,
) -> Result<()> {
    let public_dir = Arc::new(public_dir);
    let listener = TcpListener::bind(socket_addr).await?;
    loop {
        let ((stream, _)) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(io, service_fn(move |req| {
                    async move {
                        handle_request(
                            req,
                            public_dir.clone(),
                            rx_at_website.clone(),
                            tx_to_algonet.clone(),
                            tx_to_grafnet.clone(),
                            shared_state.clone(),
                        ).await
                    }
                }).await
            {
                eprintln!("failed to serve connection: {}", e);
            }
        });
    }
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body("not found".into())
        .unwrap()
}

fn bad_request() -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("bad request".into())
        .unwrap()
}

fn internal_server_error() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body("internal server error".into())
        .unwrap()
}
