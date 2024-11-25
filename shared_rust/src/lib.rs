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
    //--[ website to public_router ]--//
    RequestAbortRun(Password),
    RequestRun(Password, RunParams),
    //--[ public_router to website ]--//
    RunState(RunState),
    AlgoList(Vec<String>),
    AlgoResult(u32, AlgoResult),

    //--[ internal_router to graph_gen_container ]--//
    GraphDist(GraphDist),
    //--[ graph_gen_container to internal_router ]--//
    Graph(Graph),

    //--[ internal_router to algorithm_container ]--//
    // Graph(Graph),
    Abort,
    //--[ algorithm_container to internal_router ]--//
    SignUp(String),
    Result(AlgoResult),

    //--[ public_router to internal_router ]--//
    AbortRun,
    Run(RunParams),
    //--[ internal_router to public_router ]--//
    // AlgoList(Vec<String>),
    // AlgoResult(u32, AlgoResult),
}

pub enum WsFormat {
    Bincode,
    Json,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunState {
    running: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Password {
    inner: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlgoResult {
    result: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub serialized_matrix: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunParams {
    pub graph_dist: GraphDist,
    pub algo_ids_selected: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphDist {
    pub nodes_min: u16,
    pub nodes_max: u16,
    pub nodes_step: u16,
    pub edge_density: f64,
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

impl RunParams {
    pub fn sanitize(&mut self) {
        self.graph_gen_params.sanitize();
    }
}

impl GraphDist {
    pub fn sanitize(&mut self) {
        if (self.edge_density < 0. || self.edge_density > 1.0) {
            self.edge_density = Self::default().edge_density;
        }
    }
}

impl Default for GraphDist {
    fn default() -> Self {
        Self {
            nodes_min: 20,
            nodes_max: 2000,
            nodes_step: 20,
            edge_density: 0.2,
        }
    }
}
