use warp::{
    http::{Response, StatusCode},
    Rejection,
};

use crate::openfaas::{DefaultApi, DefaultApiClient};

pub async fn list_functions(client: DefaultApiClient) -> Result<impl warp::Reply, warp::Rejection> {
    println!("list_functions");
    let functions = client.get_functions().await.map_err(|e| {
        println!("{}", e);
        warp::reject::reject()
    })?;
    let body = serde_json::to_string(&functions).unwrap();
    Ok(Response::builder().body(body))
}
