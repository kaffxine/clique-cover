use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct SliderParams<T: std::cmp::Ord> {
    min: T,
    max: T,
    step: T,
    default: T,
}

#[derive(Serialize, Deserialize)]
struct AlgoSelect {
    name: String,
    selected: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub nodes_min: u16,
    pub nodes_max: u16,
    pub nodes_step: u16,
    pub graph_density: f64,
    pub algorithms: Vec<AlgoSelect>,
}
