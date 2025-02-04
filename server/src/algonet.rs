use futures::future;
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU16, Ordering};
use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt};
use tokio::net::{TcpStream, TcpListener};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::Message;
use twox_hash::XxHash64;
use shared::{MyMsg, Graph, Solution, Result};
use crate::types::SharedState;

pub async fn handle_algonet(
    socket_addr: SocketAddr,
    rx_at_algonet: broadcast::Receiver<MyMsg>,
    tx_to_website: broadcast::Sender<MyMsg>,
    shared_state: Arc<SharedState>,
) -> Result<()> {
    let listener = TcpListener::bind(socket_addr).await?;
    let state = AlgonetHandlerState {
        rx_at_algonet,
        tx_to_website,
        shared_state,
    };
    println!("algonet: initiated");

    loop {
        let (tcp_stream, _) = listener.accept().await?;
        let ws_stream = accept_async(tcp_stream).await?;
        let state = state.clone();
        tokio::spawn(async move {
            handle_algonet_ws(ws_stream, state).await;
        });
    }
}

struct AlgonetHandlerState {
    rx_at_algonet: broadcast::Receiver<MyMsg>,
    tx_to_website: broadcast::Sender<MyMsg>,
    shared_state: Arc<SharedState>,
}

impl Clone for AlgonetHandlerState {
    fn clone(&self) -> Self {
        Self {
            rx_at_algonet: self.rx_at_algonet.resubscribe(),
            tx_to_website: self.tx_to_website.clone(),
            shared_state: self.shared_state.clone(),
        }
    }
}

async fn handle_algonet_ws(
    ws_stream: WebSocketStream<TcpStream>,
    state: AlgonetHandlerState,
) {
    println!("algonet: handle_websocket: initiated");
    let (mut ws_write, mut ws_read) = ws_stream.split();

    let (tx_algo_id, mut rx_algo_id) = mpsc::channel(1);

    let handles: Vec<tokio::task::JoinHandle<()>> = vec![
        {
            let tx_to_website = state.tx_to_website;
            let state = state.shared_state.clone();
            tokio::spawn(async move {
                let mut algo_id = 0xffffu16;
                while let Some(Ok(Message::Binary(contents))) = ws_read.next().await {
                    let deserialized: std::result::Result<MyMsg, _> 
                        = bincode::deserialize(&contents);
                    if let Ok(msg) = deserialized {
                        match msg {
                            MyMsg::Greet(name) => {
                                println!("A new algorithm has signed up: {}", name);

                                let tx_algo_id = tx_algo_id.clone();

                                // add algorithm to state.algorithms
                                {
                                    let mut guard = state.algorithms.write().await;
                                    algo_id = guard.len() as u16;
                                    if (algo_id == 0xffff) {
                                        eprintln!("algonet: a hilarious situation has occured");
                                    }
                                    guard.push((algo_id, name));
                                }

                                // add algorithm to state.algos_in_use
                                {
                                    let mut guard = state.algos_in_use.write().await;
                                    guard.push(algo_id);
                                }

                                let _ = tx_algo_id.send(algo_id).await;

                            },
                            MyMsg::SolutionProduced(session_id, graph_id, solution) => {
                                if session_id == state.session_id.load(Ordering::Relaxed) {
                                    println!("algonet: received solution#{algo_id}#{graph_id}");

                                    state
                                        .database.read().await
                                        .insert_solution(solution, algo_id, graph_id).await;

                                    let msg = MyMsg::SolutionReady(session_id, algo_id, graph_id);
                                    tx_to_website.send(msg);
                                }
                            },
                            _ => {},
                        }
                    }
                }
            })
        },
        {
            let mut rx_at_algonet = state.rx_at_algonet;
            let state = state.shared_state.clone();
            tokio::spawn(async move {
                let mut active = false;
                let id = rx_algo_id.recv().await
                    .expect("algonet: fatal error, failed to send id");
                println!("algonet: ipc_read task has received id={id}");

                while let Ok(msg) = rx_at_algonet.recv().await {
                    match msg {
                        MyMsg::AlgosInUse(algos) => {
                            if algos.contains(&id) {
                                active = true;
                                println!("algonet: algo#{id} set as ACTIVE");
                            } else {
                                println!("algonet: algo#{id} set as INACTIVE");
                            }
                        },
                        MyMsg::GraphReady(session_id, graph_id) => {
                            if let Some(graph) = state
                                .database
                                .read().await
                                .get_graph(graph_id).await
                            {
                                println!("algonet: algo#{id}: received graph#{graph_id}");
                                let msg = MyMsg::Graph(session_id, graph_id, graph);
                                if let Ok(serialized) = bincode::serialize(&msg) {
                                    if let Err(e) = ws_write
                                        .send(Message::Binary(serialized))
                                        .await
                                    {
                                        eprintln!("algo#{id}: failed to send graph#{graph_id}");
                                    }
                                } else {
                                    eprintln!("algo#{id}: failed to serialize graph#{graph_id}");
                                }
                            } else {
                                eprintln!("algo#{id}: failed to fetch graph#{graph_id}");
                            }
                        },
                        _ => {},
                    }
                }
            })
        },
    ];

    let (_, _, tasks) = future::select_all(handles).await;

    println!("algonet_handler: aborting");
    for handle in tasks {
        handle.abort();
    }
}

