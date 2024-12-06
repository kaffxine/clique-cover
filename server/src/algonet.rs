use dashmap::DashMap;
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::hash::Hasher;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use tokio::net::{TcpStream, TcpListener};
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use twox_hash::XxHash64;
use shared::{MyMsg, AlgoResult, Graph};
use crate::types::SharedState;

pub async fn handle_algonet(
    shared_state: SharedState,
    listener: TcpListener,
    rx_at_algonet: broadcast::Receiver<MyMsg>,
    tx_to_website: broadcast::Sender<MyMsg>,
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
    let is_running = Arc::new(AtomicBool::new(false));

    let client_id = while let Some(Ok(Message::Binary(msg))) = ws_receiver.next().await {
        if let Ok(MyMsg::Greet(client_name)) = bincode::deserialize(&msg) {
            let hashed_name = {
                let mut hasher = XxHash64::with_seed(HASH_SEED);
                hasher.write(client_name.as_bytes());
                break hasher.finish();
            }
        }
    }
    let client_id = Arc::new(client_id);
    
    tokio::spawn(async move {
        let is_running = is_running.clone();
        let client_id = client_id.clone();
        let session_id = shared_state.session_id.clone();

        while let Ok(msg) = rx_at_algonet.recv().await {
            match msg {
                MyMsg::Restart(algo_list) => {
                    if (algo_list.contains(client_id) {
                        is_running.store(true, Ordering::SeqCst);
                    } else {
                        is_running.store(false, Ordering::SeqCst);
                    }
                },
                MyMsg::GraphReady(message_session_id, _) => {
                    if message_session_id != session_id.load(Ordering::SeqCst) { continue; }

                    if let Ok(msg) = bincode::serialize(&msg) {
                        let _ = ws_sender.send(Message::Binary(msg)).await;
                    }
                },
                _ => {}
            }
        }
    }

    tokio::spawn(async move {
        let is_running = is_running.clone();
        let client_id = client_id.clone();
        let session_id = shared_state.session_id.clone();

        while let Some(Ok(Message::Binary(msg))) = ws_receiver.next().await {
            if !is_running.load(Ordering::SeqCst) { continue; }

            if let Ok(msg) = bincode::deserialize(&msg) {
                match msg {
                    MyMsg::AlgoResult(result_session_id, result_graph_id, algo_result) => {
                        if result_session_id != session_id.load(Ordering::SeqCst) { continue; }

                        shared_state
                            .algo_results
                            .insert((client_id, result_graph_id, algo_result));

                        let response = MyMsg::AlgoResultReady(result_session_id, result_graph_id, client_id);
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

