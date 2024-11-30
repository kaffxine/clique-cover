use serde::{Serialize, Deserialize};
use std::io::{Read, Write};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MyMsg {
    //--[ website to public_router ]--//
    RequestRun(String, RunParams), // password
    //--[ public_router to website ]--//
    IsRunning(bool),
    AlgoList(Vec<String>), // list of algorithm names
    AlgoFinished(u8, u64, u64), // session_id, algo_id, graph_id

    //--[ internal_router to graph_gen_container ]--//
    GraphDist(u8, GraphDist), // session_id
    //--[ graph_gen_container to internal_router ]--//
    Graph(u8, Graph),

    //--[ internal_router to algorithm_container ]--//
    // Graph(Graph),
    //--[ algorithm_container to internal_router ]--//
    Greet(String), // algo_name
    AlgoResult(u8, AlgoResult), // session_id

    //--[ public_router to internal_router ]--//
    Run(Vec<u64>), // vec_of_algo_id
    //--[ internal_router to public_router ]--//
    // AlgoList(Vec<String>),
    // AlgoResult(u32, AlgoResult),
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
