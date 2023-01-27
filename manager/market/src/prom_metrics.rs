use lazy_static::lazy_static;
use prometheus::{opts, register_gauge_vec, GaugeVec};

macro_rules! PREFIX {
    () => {
        "market_"
    };
}

lazy_static! {
    pub static ref FUNCTION_DEPLOYMENT_TIME_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(
                concat!(PREFIX!(), "function_deployment_time"),
                "Time between the SLA is submitted and the function is \
                 placed."
            ),
            &["function_name", "bid_id"],
        )
        .unwrap()
    };
}
