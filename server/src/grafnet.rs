use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use shared::{MyMsg};

use crate::types::{SharedState, Database};

pub async fn handle_grafnet(
    shared_state: SharedState,
    socket_addr: SocketAddr,
    rx_at_grafnet: mpsc::Receiver<MyMsg>,
    tx_to_algonet: broadcast::Sender<MyMsg>,
) {
    while let Ok((tcp_stream, _)) = listener.accept().await {
        let 
    }
}

