use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

#[cfg(feature = "client")]
use {
    websocket::sync::stream::TcpStream,
    websocket::client::sync::Client,
    websocket::OwnedMessage,
};

#[cfg(feature = "server")]
use {
    axum::extract::ws::{self, WebSocket},
    tokio::sync::{broadcast, mpsc},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Abort,
    AlgoResult(AlgoResult),
    Start(Settings),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlgoResult {
    result: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {}

pub enum WsFormat {
    Bincode,
    Json,
}

#[cfg(feature = "client")]
impl Message {
    pub fn sync_send_via_websocket(&self, socket: &mut Client<TcpStream>, format: WsFormat) -> Result<()> {
        let msg = match format {
            WsFormat::Bincode => OwnedMessage::Binary(bincode::serialize(&self)?),
            WsFormat::Json => OwnedMessage::Text(serde_json::to_string(&self)?),
        };
        socket.send_message(&msg)?;
        Ok(())
    }
    
    pub fn sync_receive_from_websocket(socket: &mut Client<TcpStream>) -> Result<Self> {
        let msg = socket.recv_message().map_err(|e| anyhow!("socket read failed: {}", e))?;
        match msg {
            OwnedMessage::Binary(payload) => Ok(bincode::deserialize(&payload)?),
            OwnedMessage::Text(payload) => Ok(serde_json::from_str(&payload)?),
            OwnedMessage::Close(_) => Err(anyhow!("websocket gracefully closed")),
            _ => Err(anyhow!("unrecognized message format")),
        }
    }
}

#[cfg(feature = "server")]
impl Message {
    pub async fn move_via_mpsc(self, sender: &mpsc::Sender<Self>) -> Result<()> {
        sender.send(self).await.map_err(|e| anyhow!(e.to_string()))
    }

    pub async fn receive_from_mpsc(receiver: &mut mpsc::Receiver<Self>) -> Result<Self> {
        receiver.recv().await.ok_or_else(|| anyhow!("mpsc channel closed"))
    }

    pub async fn send_via_broadcast(&self, sender: &broadcast::Sender<Self>) -> Result<()> {
        sender.send(self.clone()).map_err(|e| anyhow!(e.to_string())).map(|_| ())
    }

    pub async fn receive_from_broadcast(receiver: &mut broadcast::Receiver<Self>) -> Result<Self> {
        receiver.recv().await.map_err(|e| anyhow!(e.to_string()))
    }

    pub async fn send_via_websocket(&self, socket: &mut WebSocket, format: WsFormat) -> Result<()> {
        let send_future = match format {
            WsFormat::Bincode => socket.send(ws::Message::Binary(bincode::serialize(&self)?)),
            WsFormat::Json => socket.send(ws::Message::Text(serde_json::to_string(&self)?)),
        };
        send_future.await.map_err(|e| anyhow!(e.to_string()))
    }

    pub async fn receive_from_websocket(socket: &mut WebSocket) -> Result<Self> {
        let msg = socket.recv().await.ok_or_else(|| anyhow!("websocket stream ended"))??;
        match msg {
            ws::Message::Binary(payload) => Ok(bincode::deserialize(&payload)?),
            ws::Message::Text(payload) => Ok(serde_json::from_str(&payload)?),
            ws::Message::Close(_) => Err(anyhow!("websocket gracefully closed")),
            _ => Err(anyhow!("unrecognized message format")),
        }
    }
}
