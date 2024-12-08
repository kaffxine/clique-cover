use serde::{Serialize, Deserialize};
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MyMsg {
    // website_generic_handler -> website_client_handler
    // website_client_handler -> website
    // 
    // session ID has changed
    Session(u8),

    // website_client_handler -> website
    //
    // send a list of available algorithms
    AlgoList(Vec<String>),
    
    // algonet_client_handler -> solution_database
    // solution_database -> website_client_handler
    // website_client_handler -> website
    //
    // an algorithm has produced a solution
    SolutionReady(u8, u16, u32), // session, algo, graph
    
    // algonet -> algonet_generic_handler
    // new algorithms has appeared, spawn new client_algonet_handler
    Greet(String),

    // algonet -> algonet_client_handler
    // an algorithm has produced a solution
    Solution(u8, u32, Solution), // session, graph, Solution

    // grafnet_handler -> client_
    GraphReady(u8, u32),
    Graph(u8, Graph),

    GraphDist(u8, GraphDist),
    Restart(Vec<(u64, u16)>), // hash, id
    RequestRestart(String, RunParams),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Algo {
    pub id: u16,
    pub hash: u64,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Solution {
    pub correct: bool,
    pub n_cliques: u16,
    pub n_cpu_cycles: u64,
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
    pub n_nodes_min: u16,
    pub n_nodes_max: u16,
    pub n_nodes_step: u16,
    pub node_degree: u16,
    pub n_iterations: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Summary {}

impl GraphDist {
    pub fn empty() -> Self {
        Self {
            n_nodes_min: 0,
            n_nodes_max: 0,
            n_nodes_step: 0,
            node_degree: 0,
            n_iterations: 0,
        }
    }
}

impl Default for GraphDist {
    fn default() -> Self {
        Self {
            n_nodes_min: 20,
            n_nodes_max: 2000,
            n_nodes_step: 20,
            node_degree: 2,
            n_iterations: 1,
        }
    }
}
