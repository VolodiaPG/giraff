use uom::si::f64::Information;

#[derive(Debug)]
pub struct Allocatable {
    pub cpu: String,
    pub memory: Information,
}

#[derive(Debug)]
pub struct Usage {
    pub cpu: String,
    pub memory: Information,
}

#[derive(Debug)]
pub struct Metrics {
    pub usage: Option<Usage>,
    pub allocatable: Option<Allocatable>,
}
