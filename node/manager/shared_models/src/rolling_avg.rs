use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uom::si::f64::Time;

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RollingAvg {
    #[serde_as(as = "super::DateTimeHelper")]
    last_update: DateTime<Utc>,
    #[serde_as(as = "uom_helpers::time::Helper")]
    avg: Time,
    count: u32,
}

impl RollingAvg {
    pub fn update(&mut self, now: DateTime<Utc>, emission: DateTime<Utc>) {
        let latency =
            Time::new::<uom::si::time::millisecond>((now - emission).num_milliseconds() as f64);

        self.count += 1;
        self.avg = (latency + self.avg * ((self.count - 1) as f64)) / self.count as f64;

        self.last_update = now;
    }

    pub fn get_avg(&self) -> Time {
        self.avg
    }

    pub fn get_last_update(&self) -> DateTime<Utc> {
        self.last_update
    }
}

impl Default for RollingAvg {
    fn default() -> Self {
        RollingAvg {
            last_update: Utc::now(),
            avg: Time::new::<uom::si::time::millisecond>(0.0),
            count: 0,
        }
    }
}
