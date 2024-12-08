#[allow(unused_imports)]

use bytes::Bytes;
// use futures_util::{sink::SinkExt, stream::StreamExt};
use futures::future;
use futures::{SinkExt, StreamExt, TryStreamExt};
use http_body_util::{combinators::BoxBody, BodyExt, Full, StreamBody, Empty};
use hyper::{Request, Response, StatusCode};
use hyper::body::{Body, Incoming, Frame};
use hyper::header;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::upgrade;
use hyper_util::rt::tokio::TokioIo;
use hyper_tungstenite::{WebSocketStream, HyperWebsocket};
use hyper_tungstenite::tungstenite::protocol::Message;
use serde_json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio_util::io::ReaderStream;

use shared::{Result, Error, MyMsg};

use crate::types::{SharedState, Database};

async fn handle_websocket(
    ws: HyperWebsocket,
    state: WebsiteHandlerState,
) -> Result<()> {
    println!("handle_websocket: initiated");
    let mut ws = ws.await?;
    let (mut ws_write, mut ws_read) = ws.split();
    println!("handle_websocket: split");
    
    let shared_state = state.shared_state;
    
    let handles: Vec<tokio::task::JoinHandle<()>> = vec![
        tokio::spawn(async move {
            println!("handle_websocket: started write task");
            let mut rx_at_website = state.rx_at_website;
            loop {
                sleep(Duration::from_secs(10)).await;
            }
            println!("handle_websocket: finished write task");
        }),
        tokio::spawn(async move {
            println!("handle_websocket: started read task");
            let tx_to_algonet = state.tx_to_algonet;
            let tx_to_grafnet = state.tx_to_grafnet;
            while let Some(Ok(Message::Text(contents))) = ws_read.next().await {
                println!("received from websocket: {}", contents);
                match serde_json::from_str::<MyMsg>(&contents) {
                    Ok(data) => {
                        println!("a new message: {:?}", data);
                    },
                    Err(e) => {
                        eprintln!("failed to parse JSON: {}", e);
                    },
                }
            };
            println!("handle_websocket: finished read task");
        }),
    ];

    let (_, _, tasks) = future::select_all(handles).await;
    
    println!("handle_websocket: aborting");
    
    for handle in tasks {
        handle.abort();
    }

    Ok(())
}

async fn serve_file(path: &Path) -> Result<Response<BoxBody<Bytes, Infallible>>> {
    let file = File::open(path).await;
    if file.is_err() {
        eprintln!("unable to open file");
        return not_found();
    }
    let file = file.unwrap();

    let reader_stream = ReaderStream::new(file);
    let reader_stream = reader_stream.map_err(|_| {
        unreachable!("one error in the reader stream \
            and the entire app goes down, \
            but unfortunately i dont have the time \
            to come up with a better solution");
    });
    let stream_body = StreamBody::new(reader_stream.map_ok(Frame::data));
    let boxed_body = BodyExt::boxed(stream_body);
    
    let mime_type = mime_guess::from_path(path)
        .first_or_octet_stream()
        .as_ref()
        .to_string();

    let res = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime_type)
        .body(boxed_body)
        .expect("failed to produce a response");
    Ok(res)
}

async fn handle_request(
    mut req: Request<Incoming>,
    public_dir: Arc<PathBuf>,
    state: WebsiteHandlerState,
) -> Result<Response<BoxBody<Bytes, Infallible>>> {
    let path = req.uri().path();
    println!("request for {:?} reached handle_request", path);
    match path {
        "/" => {
            let path = public_dir.clone().join("index.html");
            match serve_file(path.as_path()).await {
                Ok(res) => Ok(res),
                Err(_) => internal_server_error(),
            }
        },
        "/ws" => {
            if !hyper_tungstenite::is_upgrade_request(&req) {
                eprintln!("bad upgrade request");
                return bad_request();
            }
            let (_, ws) = hyper_tungstenite::upgrade(&mut req, None)?;
            tokio::spawn(async move {
                if let Err(e) = handle_websocket(
                    ws,
                    state,
                ).await {
                    eprintln!("failed to handle websocket");
                }
            });
            switching_protocols()
        },
        _ => {
            if path.contains("..") {
                return bad_request();
            }
            let path = path.trim_start_matches('/');
            let path = public_dir.clone().join(path);
            match serve_file(path.as_path()).await {
                Ok(res) => Ok(res),
                Err(_) => internal_server_error(),
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
    println!("website handler initiated");

    let public_dir = Arc::new(public_dir);
    let listener = TcpListener::bind(socket_addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let public_dir = public_dir.clone();
        let state = WebsiteHandlerState {
            rx_at_website: rx_at_website.resubscribe(),
            tx_to_algonet: tx_to_algonet.clone(),
            tx_to_grafnet: tx_to_grafnet.clone(),
            shared_state: shared_state.clone(),
        };
        let service = service_fn(move |req| {
            println!("request for {:?} reached service_fn", req.uri().path());

            let public_dir = public_dir.clone();
            let state = state.clone();

            async move {
                handle_request(
                    req,
                    public_dir,
                    state,
                ).await
            }
        });
        let mut conn = http1::Builder::new()
            .serve_connection(io, service)
            .with_upgrades();

        tokio::spawn(async move {
            if let Err(e) = {
                eprintln!("failed to serve connection: {}", e);
            }
        });
    }
}

struct WebsiteHandlerState {
    rx_at_website: broadcast::Receiver<MyMsg>,
    tx_to_algonet: broadcast::Sender<MyMsg>,
    tx_to_grafnet: mpsc::Sender<MyMsg>,
    shared_state: Arc<SharedState>,
}

impl Clone for WebsiteHandlerState {
    fn clone(&self) -> Self {
        Self {
            rx_at_website: self.rx_at_website.resubscribe(),
            tx_to_algonet: self.tx_to_algonet.clone(),
            tx_to_grafnet: self.tx_to_grafnet.clone(),
            shared_state: self.shared_state.clone(),
        }
    }
}

fn not_found() -> Result<Response<BoxBody<Bytes, Infallible>>> {
    Ok(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::new(Bytes::from_static(b"not found")).boxed())
        .unwrap())
}

fn bad_request() -> Result<Response<BoxBody<Bytes, Infallible>>> {
    Ok(Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body(Full::new(Bytes::from_static(b"bad request")).boxed())
        .unwrap())
}

fn internal_server_error() -> Result<Response<BoxBody<Bytes, Infallible>>> {
    Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::from_static(b"internal server error")).boxed())
        .unwrap())
}

fn switching_protocols() -> Result<Response<BoxBody<Bytes, Infallible>>> {
    Ok(Response::builder()
        .status(StatusCode::SWITCHING_PROTOCOLS)
        .header("Upgrade", "websocket")
        .header("Connection", "Upgrade")
        .header("Sec-WebSocket-Accept", "")
        .body(Empty::new().boxed())
        .unwrap())
}

