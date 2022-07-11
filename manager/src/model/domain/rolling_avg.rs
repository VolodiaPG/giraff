use serde::{Deserialize, Serialize};
use uom::si::f64::Time;

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RollingAvg {
    #[serde_as(as = "crate::helper::uom::time::Helper")]
    avg:   Time,
    count: u32,
}

/// Cumulative avg
impl RollingAvg {
    pub fn update(&mut self, latency: Time) {
        self.count += 1;
        self.avg += (latency - self.avg) / (self.count as f64);
    }

    pub fn get_avg(&self) -> Time { self.avg }
}

impl Default for RollingAvg {
    fn default() -> Self {
        RollingAvg { avg: Time::new::<uom::si::time::millisecond>(0.0), count: 0 }
    }
}
