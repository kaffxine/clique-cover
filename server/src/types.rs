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
    pub tx_to_extern: mpsc::Sender<MsgToExtern>,
    pub rx_to_extern: Arc<Mutex<Option<mpsc::Receiver<MsgToExtern>>>>,
}

#[derive(Serialize)]
pub struct Algo {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct MsgToIntern {};

#[derive(Serialize, Deserialize)]
pub struct MsgToExtern {};

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

impl AppState {
    pub fn new() -> Self {
        let (tx_int, _) = broadcast::channel::<MsgToIntern>(SIZE);
        let (tx_ext, rx_ext) = mpsc::channel::<MsgToExtern>(SIZE);
        AppState {
            run_params: Arc::new(RwLock::new(RunParams {
                graph_gen_params: GraphGenParams::default(),
                algo_ids_selected: Vec::new(),
            })),
            algo_list: Arc::new(RwLock::new(Vec::new())),
            tx_to_intern = tx_int,
            tx_to_extern = tx_ext,
            rx_to_extern = Arc::new(Mutex::new(Some(rx_ext))),
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

