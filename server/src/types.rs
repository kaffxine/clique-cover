use bincode;
use axum::extract::ws::{Message, WebSocket};
use regex::Regex;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::error::Error;
use std::sync::{Arc};
use std::sync::atomic::{self, AtomicBool};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

#[derive(Clone)]
pub struct AppState {
    pub run_params: Arc<RwLock<RunParams>>,
    pub algo_list: Arc<RwLock<Vec<Algo>>>,
    pub tx_to_intern: broadcast::Sender<MsgToIntern>,
    pub tx_to_public: mpsc::Sender<MsgToPublic>,
    pub rx_to_public: Arc<Mutex<Option<mpsc::Receiver<MsgToPublic>>>>,
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
