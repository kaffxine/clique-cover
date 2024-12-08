#[allow(unused_imports)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use tokio::net::{TcpStream, TcpListener};
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use shared::{MyMsg, Solution, Graph, GraphDist, Algo, Summary};

pub struct SharedState {
    pub session_id: AtomicU8,
    pub algorithms: RwLock<Vec<Algo>>,
    pub database: Database,
}

pub struct Database {
    size: u64,
    h_size: u16,
    v_size: u16,
    n_algos: u16,
    graphs: Box<[RwLock<Option<Graph>>]>,
    solutions: Box<[RwLock<Option<Solution>>]>,
    solution_order: RwLock<Vec<u64>>,
    summary: RwLock<Option<Summary>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            session_id: AtomicU8::new(0),
            algorithms: RwLock::new(Vec::new()),
            database: Database::new(0, GraphDist::empty()),
        }
    }
}

impl Database {
    pub fn new(n_algos: u16, graph_dist: GraphDist) -> Self {
        let v_size = (1 + graph_dist.n_nodes_max - graph_dist.n_nodes_min) / 2;
        let h_size = graph_dist.n_iterations;
        let size = v_size * h_size * n_algos;

        let graphs_vec: Vec<RwLock<Option<Graph>>> =
            (0..h_size * v_size)
                .map(|_| RwLock::new(None))
                .collect();

        let solutions_vec: Vec<RwLock<Option<Solution>>> =
            (0..size)
                .map(|_| RwLock::new(None))
                .collect();

        let solution_order = RwLock::new(Vec::<u64>::with_capacity(size.into()));

        Self {
            size: size.into(),
            h_size,
            v_size,
            n_algos,
            graphs: graphs_vec.into_boxed_slice(),
            solutions: solutions_vec.into_boxed_slice(),
            solution_order,
            summary: RwLock::new(None),
        }
    }

    pub async fn insert_graph(&self, graph: Graph, graph_id: u32) {
        if let Some(lock) = self.graphs.get(graph_id as usize) {
            let mut guard = lock.write().await;
            *guard = Some(graph);
        }
    }

    pub async fn get_graph(&self, graph_id: u32) -> Option<Graph> {
        if let Some(lock) = self.graphs.get(graph_id as usize) {
            let guard = lock.read().await;
            guard.clone()
        } else {
            None
        }
    }

    fn get_solution_index(&self, algo_id: u16, graph_id: u32) -> usize {
        (algo_id as usize) 
            * (self.h_size as usize) 
            * (self.v_size as usize) 
            + (graph_id as usize)
    }

    pub async fn insert_solution(&self, solution: Solution, algo_id: u16, graph_id: u32) {
        let index = self.get_solution_index(algo_id, graph_id);
        if let Some(lock) = self.solutions.get(index) {
            let mut guard = lock.write().await;
            *guard = Some(solution);
        }
    }

    pub async fn get_solution(&self, algo_id: u16, graph_id: u32) -> Option<Solution> {
        let index = self.get_solution_index(algo_id, graph_id);
        if let Some(lock) = self.solutions.get(index) {
            let guard = lock.read().await;
            guard.clone()
        } else {
            None
        }
    }
}
