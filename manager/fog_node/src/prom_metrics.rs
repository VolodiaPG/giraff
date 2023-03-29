use lazy_static::lazy_static;
use prometheus::{opts, register_gauge_vec, GaugeVec};

macro_rules! PREFIX {
    () => {
        "fog_node_"
    };
}

lazy_static! {
    /// Histogram of ["function_name", "bid_id"]
    pub static ref BID_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(), "bids"), "Bids placed for submitted SLA"),
            &["function_name", "bid_id", "sla_id"],
        )
        .unwrap()
    };

    pub static ref SLA_SEEN: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(), "sla_seen"), "Bids placed for submitted SLA"),
            &["function_name", "sla_id"],
        )
        .unwrap()
    };

    pub static ref MEMORY_ALLOCATABLE_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"memory_allocatable"), "Memory allocatable on fog_node"),
            &["name"],
        )
        .unwrap()
    };

    pub static ref MEMORY_USAGE_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"memory_usage"), "Memory usage on fog_node"),
                        &["name"],

        )
        .unwrap()
    };

    pub static ref CPU_ALLOCATABLE_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"cpu_allocatable"), "CPU allocatable on fog_node"),
                        &["name"],

        )
        .unwrap()
    };

     pub static ref CPU_USAGE_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"cpu_usage"), "CPU usage on fog_node"),
                        &["name"],

        )
        .unwrap()
    };

    pub static ref MEMORY_AVAILABLE_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"memory_available"), "Memory available on fog_node (from fog_node's perspective)"),
            &["name"],
        )
        .unwrap()
    };

    pub static ref MEMORY_USED_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"memory_used"), "Memory used on fog_node (from fog_node's perspective)"),
                        &["name"],

        )
        .unwrap()
    };

    pub static ref CPU_AVAILABLE_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"cpu_available"), "CPU available on fog_node (from fog_node's perspective)"),
                        &["name"],

        )
        .unwrap()
    };

     pub static ref CPU_USED_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"cpu_used"), "CPU used on fog_node (from fog_node's perspective)"),
                        &["name"],

        )
        .unwrap()
    };

     pub static ref LATENCY_NEIGHBORS_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"neighbors_latency"), "Latency with neighbors (parent & children)"),
                        &["instance_to"],

        )
        .unwrap()
    };

    pub static ref LATENCY_NEIGHBORS_AVG_GAUGE: GaugeVec = {
        register_gauge_vec!(
            opts!(concat!(PREFIX!(),"neighbors_latency_rolling_avg"), "Latency with neighbors (parent & children) average computed on the node"),
                        &["instance_to"],

        )
        .unwrap()
    };
}
