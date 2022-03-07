use validator::Validate;
use warp::{
    http::{Response, StatusCode},
    Rejection,
};
use k8s_openapi::api::core::v1::{Event, Node};
use kube::{
    api::{Api, ListParams, ResourceExt},
    Client,
};

use sla::SLA;

use crate::openfaas::{DefaultApi, DefaultApiClient};

pub async fn list_functions(client: DefaultApiClient) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("list_functions");
    let functions = client.get_functions().await.map_err(|e| {
        error!("{}", e);
        warp::reject::reject()
    })?;
    let body = serde_json::to_string(&functions).unwrap();
    Ok(Response::builder().body(body))
}


pub async fn post_sla(_client: DefaultApiClient, sla: SLA) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("pos sla: {:?}", sla);

    let body = serde_json::to_string(&sla).unwrap();
    Ok(Response::builder().body(body))}
