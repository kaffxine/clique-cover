use bincode;
use axum::extract::ws::{Message, WebSocket};
use regex::Regex;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::error::Error;
use std::sync::{Arc};
use std::sync::atomic::{self, AtomicBool};
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};

const SIZE: u32 = 256;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub serialized_matrix: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunParams {
    pub graph_gen_params: GraphGenParams,
    pub algo_ids_selected: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphGenParams {
    pub nodes_min: u16,
    pub nodes_max: u16,
    pub nodes_step: u16,
    pub edge_density: f64,
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
