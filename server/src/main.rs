#[allow(unused_imports)]

use std::path::PathBuf;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};

use shared::MyMsg;

mod types;
use types::{SharedState, Database};

// mod algonet;
// mod grafnet;
mod website;
use website::handle_website;


const HASH_SEED: u64 = 0xdb2137db;
const CHANNEL_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let public_dir = PathBuf::from("/app/public");

    let website_addr: SocketAddr = ([0, 0, 0, 0], 3000).into();
    let algonet_addr: SocketAddr = ([127, 0, 0, 1], 3001).into();
    let grafnet_addr: SocketAddr = ([127, 0, 0, 1], 3002).into();

    let (tx_to_algonet, mut rx_at_algonet) = broadcast::channel::<MyMsg>(CHANNEL_SIZE);
    let (tx_to_website, mut rx_at_website) = broadcast::channel::<MyMsg>(CHANNEL_SIZE);
    let (tx_to_grafnet, mut rx_at_grafnet) = mpsc::channel::<MyMsg>(CHANNEL_SIZE);

    let shared_state = Arc::new(SharedState::new());
    
    handle_website(
        website_addr,
        public_dir,
        rx_at_website,
        tx_to_algonet.clone(),
        tx_to_grafnet.clone(),
        shared_state.clone(),
    ).await;

    Ok(())
}
