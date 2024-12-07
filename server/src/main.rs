use dashmap::DashMap;
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::hash::Hasher;
use std::net::SocketAddr;
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
use shared::{MyMsg, Solution, Graph, GraphDist, Algo};

mod types;
use types::{SharedState, Database};

mod algonet;
mod grafnet;
mod website;


const HASH_SEED: u64 = 0xdb2137db;
const CHANNEL_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let website_addr: SocketAddr = ([0, 0, 0, 0], 3000).into();
    let algonet_addr: SocketAddr = ([127, 0, 0, 1], 3001).into();
    let grafnet_addr: SocketAddr = ([127, 0, 0, 1], 3002).into();

    let (tx_to_algonet, rx_at_algonet) = broadcast::channel<MyMsg>(CHANNEL_SIZE);
    let (tx_to_website, rx_at_website) = broadcast::channel<MyMsg>(CHANNEL_SIZE);
    let (tx_to_grafnet, rx_at_grafnet) = mpsc::channel<MyMsg>(CHANNEL_SIZE);
    let shared_state = Arc::new(SharedState::new());

}
