use anyhow::{anyhow, Result};
use std::env;
use std::sync::{Arc, Mutex};
use websocket::OwnedMessage;
use websocket::client::sync::Client;
use websocket::client::builder::ClientBuilder;
use shared::MyMsg;

const URL: &str = "ws://127.0.0.1:8080";

fn main() -> Result<()> {
    let (mut ws_receiver, mut ws_sender) = ClientBuilder::new(URL)
        .expect("invalid web socket url")
        .connect_insecure()
        .expect("failed to connect to {URL}")
        .split()
        .expect("failed to split websocket client into sender and receiver");

    let algo_name = env::var("ALGO_NAME")?;
    {
        let msg = MyMsg::Greet(algo_name);
        let msg_bin = bincode::serialize(&msg)
            .expect("failed to serialize greeting");
        ws_sender
            .send_message(&OwnedMessage::Binary(msg_bin))
            .expect("failed to send greeting");
    }

    Ok(())
}
