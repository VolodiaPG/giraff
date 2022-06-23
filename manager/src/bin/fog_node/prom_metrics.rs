use lazy_static::lazy_static;
use rocket_prometheus::prometheus::{histogram_opts, HistogramVec};

lazy_static! {
    /// Histogram of ["function_name"]
    pub static ref BID_HISTOGRAM: HistogramVec = {
        HistogramVec::new(
            histogram_opts!("bids", "Bids placed for submitted SLA"),
            &["function_name"],
        )
        .unwrap()
    };
}
