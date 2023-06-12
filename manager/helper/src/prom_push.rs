use anyhow::{bail, Context, Result};
use nutype::nutype;
use prometheus::{
    labels, proto, BasicAuthentication, Encoder, ProtobufEncoder,
};
use reqwest::header::CONTENT_TYPE;
use reqwest::{Method, StatusCode, Url};
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

type HttpClient = reqwest_middleware::ClientWithMiddleware;

#[nutype(validate(with = validate_ip_port))]
#[derive(Clone, Debug, Deserialize)]
pub struct PrometheusAddress(String);
#[nutype(validate(with = validate_ip_port))]
#[derive(Clone, Debug)]
pub struct FogNodeAddress(String);

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

pub async fn send_metrics(
    http_client: Arc<HttpClient>,
    instance: String,
    prometheus_address: PrometheusAddress,
) -> Result<()> {
    let address = prometheus_address.into_inner();
    push(
        &http_client,
        "fog_node",
        labels! {"instance".to_owned() => instance},
        &address,
        prometheus::gather(),
        "PUT",
        None,
    )
    .await
    .with_context(|| format!("Pusing metrics to {}", address))
}

const LABEL_NAME_JOB: &str = "job";

async fn push<S: BuildHasher>(
    http_client: &HttpClient,
    job: &str,
    grouping: HashMap<String, String, S>,
    url: &str,
    mfs: Vec<proto::MetricFamily>,
    method: &str,
    basic_auth: Option<BasicAuthentication>,
) -> Result<()> {
    // Suppress clippy warning needless_pass_by_value.
    let grouping = grouping;

    let mut push_url = if url.contains("://") {
        url.to_owned()
    } else {
        format!("http://{}", url)
    };

    if push_url.ends_with('/') {
        push_url.pop();
    }

    let mut url_components = Vec::new();
    if job.contains('/') {
        bail!("job contains '/': {}", job);
    }

    // TODO: escape job
    url_components.push(job.to_owned());

    for (ln, lv) in &grouping {
        // TODO: check label name
        if lv.contains('/') {
            bail!("value of grouping label {} contains '/': {}", ln, lv);
        }
        url_components.push(ln.to_owned());
        url_components.push(lv.to_owned());
    }

    push_url =
        format!("{}/metrics/job/{}", push_url, url_components.join("/"));

    let encoder = ProtobufEncoder::new();
    let mut buf = Vec::new();

    for mf in mfs {
        // Check for pre-existing grouping labels:
        for m in mf.get_metric() {
            for lp in m.get_label() {
                if lp.get_name() == LABEL_NAME_JOB {
                    bail!(
                        "pushed metric {} already contains a job label",
                        mf.get_name()
                    );
                }
                if grouping.contains_key(lp.get_name()) {
                    bail!(
                        "pushed metric {} already contains grouping label {}",
                        mf.get_name(),
                        lp.get_name()
                    );
                }
            }
        }
        // Ignore error, `no metrics` and `no name`.
        let _ = encoder.encode(&[mf], &mut buf);
    }

    let mut builder = http_client
        .request(
            Method::from_str(method).unwrap(),
            Url::from_str(&push_url).unwrap(),
        )
        .header(CONTENT_TYPE, encoder.format_type())
        .body(buf);

    if let Some(BasicAuthentication { username, password }) = basic_auth {
        builder = builder.basic_auth(username, Some(password));
    }

    let response = builder.send().await.context("Request failed")?;

    match response.status() {
        StatusCode::ACCEPTED => Ok(()),
        StatusCode::OK => Ok(()),
        _ => bail!(
            "unexpected status code {} while pushing to {}",
            response.status(),
            push_url
        ),
    }
}
