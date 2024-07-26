use uom::si::rational64::{Information, Ratio};

#[derive(Debug)]
pub struct Allocatable {
    pub cpu:    Ratio,
    pub memory: Information,
}

#[derive(Debug)]
pub struct Usage {
    pub cpu:    Ratio,
    pub memory: Information,
}

#[derive(Debug)]
pub struct Metrics {
    pub usage:       Option<Usage>,
    pub allocatable: Option<Allocatable>,
}
