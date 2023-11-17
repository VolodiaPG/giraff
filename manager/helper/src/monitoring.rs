use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use convert_case::{Case, Casing};
use futures::stream::{self};
use influxdb2::models::WriteDataPoint;
use influxdb2::Client;
use nutype::nutype;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;

#[nutype(derive(Clone, Debug), sanitize(with = to_snake), validate(char_len_min = 3, char_len_max = 64, not_empty))]
pub struct InfluxName(String);

#[nutype(
    derive(Clone, Debug),
    validate(char_len_min = 3, char_len_max = 64, not_empty)
)]
pub struct InfluxBucket(String);

#[nutype(
    derive(Clone, Debug),
    validate(
        char_len_min = 88,
        not_empty,
        char_len_max = 88,
        regex = "^(?:[A-Za-z0-9_-]{4})*(?:\
                 [A-Za-z0-9_-][AQgw]==|[A-Za-z0-9_-]{2}[AEIMQUYcgkosw048]=)?$"
    )
)]
pub struct InfluxToken(String);

#[nutype(
    derive(Clone, Debug),
    validate(char_len_min = 3, char_len_max = 64, not_empty)
)]
pub struct InfluxOrg(String);

pub fn to_snake(data: String) -> String { data.to_case(Case::Snake) }

#[nutype(derive(Clone, Debug, Deserialize), validate(predicate = validate_ip_port))]
pub struct InfluxAddress(String);

#[nutype(derive(Clone, Debug, Deserialize), sanitize(with = to_snake), validate(char_len_min = 3, char_len_max = 64, not_empty))]
pub struct InstanceName(String);

#[derive(Debug)]
pub struct MetricsExporter {
    database: Client,
    instance: InstanceName,
    bucket:   InfluxBucket,
}

/// This function is used as an indirect way to embed the timestamp in the
/// wanted time-precision in a centralized manner from this file, instead of
/// indicating it each time a new metric is defined
pub fn convert_timestamp(timestamp: DateTime<Utc>) -> i64 {
    timestamp.timestamp_millis()
}

pub trait InfluxData {
    fn export(
        self,
        instance: String,
    ) -> impl WriteDataPoint + Sync + Send + 'static;
}

fn validate_ip_port(input: &str) -> bool {
    let parts = input.split(':');
    let collection = parts.collect::<Vec<&str>>();
    if collection.len() != 2 {
        return false;
    }

    if IpAddr::from_str(collection.first().unwrap()).is_err() {
        return false;
    }

    match collection.get(1).unwrap().parse::<usize>() {
        Ok(port) => {
            if port > 0 && port < 65536 {
                return true;
            }
            false
        }
        Err(_) => false,
    }
}

impl MetricsExporter {
    pub async fn new(
        address: InfluxAddress,
        org: InfluxOrg,
        token: InfluxToken,
        bucket: InfluxBucket,
        instance: InstanceName,
    ) -> Result<Self> {
        let ret = Self {
            database: Client::new(
                format!("http://{}", address.clone().into_inner()),
                org.clone().into_inner(),
                token.clone().into_inner(),
            ),
            instance,
            bucket: bucket.clone(),
        };
        timeout(Duration::from_secs(1), ret.database.health())
            .await
            .context("Database health request timed out")?
            .context("Database health request failed")?;
        timeout(Duration::from_secs(1), ret.database.ready())
            .await
            .context("Database timed out waiting to be ready")?
            .context("Database is not ready")?;
        timeout(Duration::from_secs(1), ret.database.is_onboarding_allowed())
            .await
            .context("Onboard check timed out")?
            .context("Onboard check failed")?;

        Ok(ret)
    }

    pub async fn observe(
        &self,
        // data: impl Stream<Item = impl WriteDataPoint> + Sync + Send +
        // 'static,
        data: impl InfluxData,
    ) -> Result<()> {
        let toto = vec![data.export(self.instance.clone().into_inner())];
        let s = stream::iter(toto);

        self.database
            .write_with_precision(
                &self.bucket.clone().into_inner(),
                s,
                influxdb2::api::write::TimestampPrecision::Milliseconds,
            )
            .await
            .context("Failed to write to influxdb2 database")?;
        Ok(())
    }
}
