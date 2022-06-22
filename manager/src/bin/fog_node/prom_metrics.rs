use lazy_static::lazy_static;
use rocket_prometheus::prometheus::{opts, HistogramVec};

lazy_static! {
    /// Histogram of ["function_name"]
    pub static ref BID_HISTOGRAM: HistogramVec = {
        HistogramVec::new(
            opts!("bids", "Bids placed for submitted SLA"),
            &["function_name"],
        )
        .unwrap()
    };
}
