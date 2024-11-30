use dashmap::DashMap;
use futures_util::{sink::SinkExt, stream::StreamExt};
use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::net::{TcpStream, TcpListener};
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use shared::{MyMsg, AlgoResult, Graph};

#[derive(Clone)]
struct SharedState {
    session_id: Arc<AtomicU8>,
    graphs: Arc<DashMap<u64, Graph>>,
    algo_names: Arc<DashMap<u64, String>>,
    algo_results: Arc<DashMap<(u64, u64), AlgoResult>>,
}

const CHANNEL_SIZE: usize = 1024;
const WEBSITE_ADDR: &str = "0.0.0.0:3000";
const ALGONET_ADDR: &str = "127.0.0.1:3001";
const GRAFNET_ADDR: &str = "127.0.0.1:3002";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx_to_algonet, rx_at_algonet) = broadcast::channel<MyMsg>(CHANNEL_SIZE);
    let (tx_to_website, rx_at_website) = broadcast::channel<MyMsg>(CHANNEL_SIZE);
    let (tx_to_grafnet, rx_at_grafnet) = mpsc::channel<MyMsg>(CHANNEL_SIZE);
    let shared_state = SharedState { Arc::new(DashMap::new()) };

    let algonet_tcp_listener = TcpListener::bind(ALGONET_ADDR).await?; 
    tokio::spawn(handle_algonet(
        algonet_tcp_listener,
        tx_to_website,
        rx_at_algonet,
        shared_state.clone(),
    ));
}

async fn handle_grafnet(
    listener: TcpListener,
    tx_to_algonet: broadcast::Sender<MyMsg>,
    rx_at_grafnet: mpsc::Receiver<MyMsg>,
    shared_state: SharedState,
) {
    while let Ok((tcp_stream, _)) = listener.accept().await {
        let ws_stream = accept_async(tcp_stream).await;
        if let Ok(ws_stream) = ws_stream {
        }
    }
}

async fn handle_algonet(
    listener: TcpListener,
    tx_to_website: broadcast::Sender<MyMsg>,
    rx_at_algonet: broadcast::Receiver<MyMsg>,
    shared_state: SharedState,
) {
    while let Ok((tcp_stream, _)) = listener.accept().await {
        let ws_stream = accept_async(tcp_stream).await;
        if let Ok(ws_stream) = ws_stream {
            tokio::spawn(handle_algonet_ws(
                ws_stream,
                tx_to_website.clone(),
                rx_at_algonet.resubscribe(),
                shared_state.clone(),
            );
        }
    }
}

async fn handle_algonet_ws(
    ws_stream: WebSocketStream<TcpStream>,
    tx_to_website: broadcast::Sender<MyMsg>,
    rx_at_algonet: broadcast::Receiver<MyMsg>,
    shared_state: SharedState,
) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let client_id = while let Some(Ok(Message::Binary(msg))) = ws_receiver.next().await {
        if let Ok(MyMsg::Greet(client_name)) = bincode::deserialize(&msg) {
            // TODO generate a hash using twox-hash
        }
    }
    
    // TODO maybe some AtomicBool "running"?

    tokio::spawn(async move {
        while let Ok(msg) = rx_at_algonet.recv().await {
            match msg {
                MyMsg::Graph(_) => {
                    // TODO Start enum variant
                    if let Ok(msg) = bincode::serialize(&msg) {
                        let _ = ws_sender.send(Message::Binary(msg)).await;
                    }
                },
                _ => {}
            }
        }
    }

    tokio::spawn(async move {
        while let Some(Ok(Message::Binary(msg))) = ws_receiver.next().await {
            if let Ok(msg) = bincode::deserialize(&msg) {
                match msg {
                    // TODO AlgoResult api changed
                    MyMsg::AlgoResult(graph_id, algo_result) => {
                        shared_state
                            .algo_results
                            .insert((client_id, graph_id, algo_result));
                        let response = MyMsg::AlgoFinished(client_id, graph_id);
                        if let Ok(response) = bincode::serialize(&response) {
                            let _ = tx_to_website.send(response);
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
