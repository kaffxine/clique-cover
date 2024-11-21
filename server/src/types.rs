use axum::extract::ws::{Message, WebSocket};
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::sync::{Arc};
use std::sync::atomic::{self, AtomicBool};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

const SIZE: u32 = 256;

#[derive(Clone)]
pub struct AppState {
    pub run_params: Arc<RwLock<RunParams>>,
    pub algo_list: Arc<RwLock<Vec<Algo>>>,
    pub tx_to_intern: broadcast::Sender<MsgToIntern>,
    pub tx_to_public: mpsc::Sender<MsgToPublic>,
    pub rx_to_public: Arc<Mutex<Option<mpsc::Receiver<MsgToPublic>>>>,
}

#[derive(Serialize)]
pub struct Algo {
    pub id: u32,
    pub name: String,
}

#[derive(Deserialize)]
pub struct RunParams {
    pub graph_gen_params: GraphGenParams,
    pub algo_ids_selected: Vec<u32>,
}

#[derive(Deserialize)]
pub struct GraphGenParams {
    pub nodes_min: u16,
    pub nodes_max: u16,
    pub nodes_step: u16,
    pub edge_density: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Msg {
    command: String,
    payload: String,
}

impl AppState {
    pub fn new() -> Self {
        let (tx_int, _) = broadcast::channel::<Msg>(SIZE);
        let (tx_ext, rx_ext) = mpsc::channel::<Msg>(SIZE);
        AppState {
            run_params: Arc::new(RwLock::new(RunParams {
                graph_gen_params: GraphGenParams::default(),
                algo_ids_selected: Vec::new(),
            })),
            algo_list: Arc::new(RwLock::new(Vec::new())),
            tx_to_intern = tx_int,
            tx_to_public = tx_ext,
            rx_to_public = Arc::new(Mutex::new(Some(rx_ext))),
        }
    }
}

impl RunParams {
    pub fn sanitize(&mut self) {
        self.graph_gen_params.sanitize();
    }
}

impl GraphGenParams {
    pub fn sanitize(&mut self) {
        if (self.edge_density < 0. || self.edge_density > 1.0) {
            self.edge_density = Self::default().edge_density;
        }
    }
}

impl Default for GraphGenParams {
    fn default() -> Self {
        Self {
            nodes_min: 20,
            nodes_max: 2000,
            nodes_step: 20,
            edge_density: 0.2,
        }
    }
}

impl Msg {
    fn pack(&self) -> String {
        format!("\\{{{}}}{}", self.command, self.payload);
    }

    fn unpack(input_str: &str) -> Result<Self, String> {
        let re = Regex::new(r"\\\{([^}]+)\}(.*)").map_err(|e| e.to_string())?;
        if let Some(cpt) = re.captures(input_str) {
            let command = cpt.get(1).map_or("", |m| m.as_str()).to_string();
            let payload = cpt.get(2).map_or("", |m| m.as_str()).to_string();
            Ok(Self { command, payload })
        } else {
            Err("Invalid format".to_string())
        }
    }

    fn busy() -> Self {
        Self {
            command: "BUSY".to_string(),
            payload: "".to_string(),
        }
    }
}
