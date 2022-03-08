use std::error::Error;

use k8s_openapi::api::core::v1::{Event, Node};
use kube::{
    api::{Api, ListParams, ResourceExt},
    Client,
};
use kube_metrics::{pod::PodMetrics, node::NodeMetrics};
use validator::Validate;
use warp::{
    http::{Response, StatusCode},
    Rejection,
};

use sla::SLA;

use crate::openfaas::{DefaultApi, DefaultApiClient};
use crate::Error::Kube;

pub async fn list_functions(client: DefaultApiClient) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("list_functions");
    let functions = client.get_functions().await.map_err(|e| {
        error!("{}", e);
        warp::reject::reject()
    })?;
    let body = serde_json::to_string(&functions).unwrap();
    Ok(Response::builder().body(body))
}

pub async fn post_sla(client: DefaultApiClient, sla: SLA) -> Result<impl warp::Reply, Rejection> {
    trace!("pos sla: {:?}", sla);

    let client = Client::try_default().await.map_err(Kube)?;
    let nodes: Api<NodeMetrics> = Api::all(client.clone());
    let metrics = nodes.list(&ListParams::default()).await.map_err(Kube)?;
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(serde_json::to_string(&metrics).unwrap()))
}
