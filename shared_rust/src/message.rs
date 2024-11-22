use anyhow::{anyhow, Result};
use axum::extract::ws::{self, WebSocket};
use serde::{Serialize, Deserialize};
use tokio::sync::{broadcast, mpsc};

    #[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message<T> {
    pub command: String,
    pub content: T,
}

impl<T> Message<T>
where
    T: Send + 'static,
{
    pub async fn move_via_mpsc(self, sender: &mpsc::Sender<Self>) -> Result<()> {
        sender.send(self).await.map_err(|e| anyhow!(e.to_string()))
    }

    pub async fn receive_from_mpsc(receiver: &mut mpsc::Receiver<Self>) -> Result<Self> {
        receiver.recv().await.ok_or_else(|| anyhow!("mpsc channel closed"))
    }
}

impl<T> Message<T>
where
    T: Clone + Send + 'static,
{
    pub async fn send_via_broadcast(&self, sender: &broadcast::Sender<Self>) -> Result<()> {
        sender.send(self.clone()).map_err(|e| anyhow!(e.to_string())).map(|_| ())
    }

    pub async fn receive_from_broadcast(receiver: &mut broadcast::Receiver<Self>) -> Result<Self> {
        Ok(receiver.recv().await?)
    }
}

impl<T> Message<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    pub async fn send_via_websocket(&self, websocket: &mut WebSocket) -> Result<()> {
        let data = bincode::serialize(&self)?;
        websocket
            .send(ws::Message::Binary(data))
            .await
            .map_err(|e| anyhow!(e.to_string()))
    }

    pub async fn receive_from_websocket(websocket: &mut WebSocket) -> Result<Self> {
        let msg = websocket.recv().await.ok_or_else(|| anyhow!("websocket stream ended"))?;
        match msg {
            Ok(ws::Message::Binary(payload)) => Ok(bincode::deserialize(&payload)?),
            Ok(ws::Message::Close(_)) => Err(anyhow!("websocket gracefully closed")),
            Ok(_) => Err(anyhow!("non-binary message")),
            Err(e) => Err(anyhow!(e.to_string())),
        }
    }
}
