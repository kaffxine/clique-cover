use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use tokio::net::{TcpStream, TcpListener};
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use shared::{MyMsg, Solution, Graph, GraphDist, Algo};

#[derive(Clone)]
pub struct SharedState {
    pub session_id: AtomicU8,
    pub database: Database,
}

#[derive(Clone)]
pub struct Database {
    size: u64,
    h_size: u16,
    v_size: u16,
    n_algos: u16,
    graphs: Box<[RwLock<Option<Graph>>]>,
    solutions: Box<[RwLock<Option<Solution>>]>,
    solution_order: RwLock<Vec<u64>>>,
}

impl SharedState {
    pub fn new(n_algos: u16, graph_dist: GraphDist)
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

        let solutions_vec: Vec<RwLock<Option<Graph>>> =
            (0..size)
                .map(|_| RwLock::new(None))
                .collect();

        let solution_order = RwLock::new(Vec::<Option<Graph>>::with_capacity(size));

        Self {
            size,
            h_size,
            v_size,
            n_algos,
            graphs: graphs_vec.into_boxed_slice(),
            solutions: solutions_vec.into_boxed_slice(),
            solution_order,
        }
    }

    pub async fn insert_graph(&self, graph: Graph, graph_id: u32) {
        if let Some(lock) = self.solutions.get(graph_id as usize) {
            let mut guard = lock.write().await;
            *guard = graph;
        }
    }

    pub async fn get_graph(&self, graph_id: u32) -> Graph {
        if let Some(lock) = self.solutions.get(graph_id as usize) {
            let guard = lock.read().await;
            *guard
        }
    }

    fn get_solution_index(&self, algo_id, graph_id) -> usize {
        algo_id * self.h_size * self.v_size + graph_id
    }

    pub async fn insert_solution(&self, solution: Solution, algo_id: u16, graph_id: u32) {
        let index = self.get_solution_index(algo_id, graph_id);
        if let Some(lock) = self.solutions.get(index) {
            let mut guard = lock.write().await;
            *guard = solution;
        }
    }

    pub async fn get_solution(&self, algo_id: u16, graph_id: u32) -> Solution {
        let index = self.get_solution_index(algo_id, graph_id);
        if let Some(lock) = self.solutions.get(index) {
            let guard = lock.read().await;
            *guard
        }
    }
}
