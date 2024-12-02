use serde::{Serialize, Deserialize};
use std::io::{Read, Write};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MyMsg {
    IsRunning(bool),

    Greet(String), // algo_name
                   //
    AlgoList(Vec<String>), // list of algorithm names
                           //
    AlgoResult(u8, u64, AlgoResult), // session_id, graph_id
    AlgoResultReady(u8, u64, u64), // session_id, graph_id, algo_id
                                //
    GraphDist(u8, GraphDist), // session_id
    Graph(u8, Graph), // session_id
    GraphReady(u8, u64), // session_id, graph_id
                                     //
    Restart(Vec<u64>), // vec_of_algo_id
    RequestRestart(String, RunParams), // password
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlgoResult {
    pub graph_hash: u64,
    pub output: String, // to be changed
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graph {
    pub inner: Vec<u8>,
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
    pub node_degree: u16,
}

impl Default for GraphDist {
    fn default() -> Self {
        Self {
            nodes_min: 20,
            nodes_max: 2000,
            nodes_step: 20,
            node_degree: 2,
        }
    }
}
