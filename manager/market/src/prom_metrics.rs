use lazy_static::lazy_static;
use prometheus::{
    opts, register_counter_vec, register_gauge_vec, CounterVec, GaugeVec,
};

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
    pub static ref PROVISIONED_FUNCTIONS_COUNT: CounterVec = {
        register_counter_vec!(
            opts!(
                concat!(PREFIX!(), "provisioned_functions"),
                "Number of provisioned functions."
            ),
            &[]
        )
        .unwrap()
    };
    pub static ref REFUSED_FUNCTIONS_COUNT: CounterVec = {
        register_counter_vec!(
            opts!(
                concat!(PREFIX!(), "refused_functions"),
                "Number of aborted/refused functions."
            ),
            &[],
        )
        .unwrap()
    };
}
