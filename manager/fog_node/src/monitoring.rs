use helper_derive::influx_observation;

/// Bids placed for submitted SLA
#[influx_observation]
struct BidGauge {
    #[influxdb(field)]
    bid:           f64,
    #[influxdb(tag)]
    function_name: String,
    #[influxdb(tag)]
    bid_id:        String,
    #[influxdb(tag)]
    sla_id:        String,
}

/// SLA that passed here
#[influx_observation]
struct SlaSeen {
    #[influxdb(field)]
    n: u64,
    #[influxdb(field)]
    accumulated_latency_median: f64,
    #[influxdb(field)]
    accumulated_latency_median_uncertainty: f64,
    #[influxdb(tag)]
    function_name: String,
    #[influxdb(tag)]
    sla_id: String,
}

/// Memory observed from the underlying platform
#[influx_observation]
struct MemoryObservedFromPlatform {
    #[influxdb(field)]
    allocatable: f64,
    #[influxdb(field)]
    used:        f64,
    #[influxdb(tag)]
    name:        String,
}

/// CPU observed from the underlying platform
#[influx_observation]
struct CpuObservedFromPlatform {
    #[influxdb(field)]
    allocatable: f64,
    #[influxdb(field)]
    used:        f64,
    #[influxdb(tag)]
    name:        String,
}

/// Memory observed from the underlying fog node
#[influx_observation]
struct MemoryObservedFromFogNode {
    #[influxdb(field)]
    initial_allocatable: f64,
    #[influxdb(field)]
    used:                f64,
    #[influxdb(tag)]
    name:                String,
}

/// CPU observed from the underlying fog node
#[influx_observation]
struct CpuObservedFromFogNode {
    #[influxdb(field)]
    initial_allocatable: f64,
    #[influxdb(field)]
    used:                f64,
    #[influxdb(tag)]
    name:                String,
}

/// Latency with neighbors (parent & children)
#[influx_observation]
struct NeighborLatency {
    #[influxdb(field)]
    raw:                 f64,
    #[influxdb(field)]
    average:             f64,
    #[influxdb(field)]
    median:              f64,
    #[influxdb(field)]
    interquartile_range: f64,
    #[influxdb(tag)]
    instance_to:         String,
    #[influxdb(tag)]
    instance_address:    String,
}

/// Number of provisioned functions
#[influx_observation]
struct ProvisionedFunctions {
    #[influxdb(field)]
    n:             u64,
    #[influxdb(tag)]
    function_name: String,
    #[influxdb(tag)]
    sla_id:        String,
}
