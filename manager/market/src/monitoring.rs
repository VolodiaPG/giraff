use helper_derive::influx_observation;

/// What is the duration of the auctioning process, from start to having a
/// candidate + duration of the provisioning phase?
#[influx_observation]
struct FunctionDeploymentDuration {
    #[influxdb(field)]
    value:                   i64,
    #[influxdb(tag)]
    function_name:           String,
    #[influxdb(tag)]
    timestamp_auction_start: String,
    #[influxdb(tag)]
    bid_id:                  String,
    #[influxdb(tag)]
    sla_id:                  String,
}

/// Number of provisioned functions.d
#[influx_observation]
struct ProvisionedFunctionGauge {
    #[influxdb(field)]
    value:         u64,
    #[influxdb(tag)]
    function_name: String,
    #[influxdb(tag)]
    sla_id:        String,
}

/// Number of aborted/refused functions.
#[influx_observation]
struct RefusedFunctionGauge {
    #[influxdb(field)]
    value:         u64,
    #[influxdb(tag)]
    function_name: String,
    #[influxdb(tag)]
    sla_id:        String,
}

/// Number of aborted/refused functions.
#[influx_observation]
struct Toto {
    #[influxdb(field)]
    value: f64,
    #[influxdb(tag)]
    toto:  String,
}
