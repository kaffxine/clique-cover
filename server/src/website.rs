use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use shared::{MyMsg};

use crate::types::{SharedState, Database};

pub async fn handle_website(
    shared_state: Arc<SharedState>,
    listener: TcpListener,
    rx_at_website: broadcast::Receiver<MyMsg>,
    tx_to_algonet: broadcast::Sender<MyMsg>,
    tx_to_grafnet: mpsc::Sender<MyMsg>,
) {
    while let Ok((tcp_stream, _)) = listener.accept().await {

    }
}
