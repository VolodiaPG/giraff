use warp::{http::Response, Rejection};

use node_logic::satisfiability::is_satisfiable;
use sla::Sla;

use crate::openfaas::{models::Satisfiable, DefaultApi, DefaultApiClient};

pub async fn list_functions(client: DefaultApiClient) -> Result<impl warp::Reply, warp::Rejection> {
    trace!("list_functions");
    let functions = client.get_functions().await.map_err(|e| {
        error!("{}", e);
        warp::reject::reject()
    })?;
    let body = serde_json::to_string(&functions).unwrap();
    Ok(Response::builder().body(body))
}

pub async fn post_sla(client: DefaultApiClient, sla: Sla) -> Result<impl warp::Reply, Rejection> {
    trace!("pos sla: {:?}", sla);

    match is_satisfiable(&sla).await {
        Ok(res) => Ok(Response::builder().body(serde_json::to_string(&Satisfiable {
            is_satisfiable: res,
            sla: Some(sla),
        }).unwrap())),
        Err(e) => {
            error!("{}", e);
            Err(warp::reject::custom(crate::Error::NodeLogicSatisfiability(
                e,
            )))
        }
    }
}
