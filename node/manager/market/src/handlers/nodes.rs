use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;
use warp::{http::Response, Rejection};

use crate::live_store::NodesDataBase;
use crate::Error;
use shared_models::node::{PostNode, PostNodeResponse};
use if_chain::if_chain;
/// Register a new node in the database
// pub async fn put_nodes(
//     nodes_db: Arc<Mutex<NodesDataBase>>,
//     payload: RegisterNode,
// ) -> Result<impl warp::Reply, Rejection> {
//     trace!("put/register a new node");

//     let node = NodeRecord {
//         ip: payload.ip,
//         ..Default::default()
//     };

//     let id;

//     {
//         id = nodes_db.lock().await.insert(node);
//     }

//     Ok(Response::builder().body(id.to_string()))
// }

/// Patch part of the node data
pub async fn post_nodes(
    nodes_db: Arc<Mutex<NodesDataBase>>,
    payload: PostNode,
) -> Result<impl warp::Reply, Rejection> {
    trace!("patch the node @{}", &payload.from);
    nodes_db
        .lock()
        .await
        .get_mut(&payload.from)
        .ok_or_else(|| warp::reject::custom(Error::NodeIdNotFound(payload.from.clone())))?
        .latency_to_market
        .update(Utc::now(), payload.created_at);

    if_chain! {
        if let Some(last_answered_at) = payload.last_answered_at;
        if let Some(last_answer_received_at) = payload.last_answer_received_at;
        then {
            nodes_db
                .lock()
                .await
                .get_mut(&payload.from)
                .ok_or_else(|| warp::reject::custom(Error::NodeIdNotFound(payload.from.clone())))?
                .latency_to_node
                .update(last_answer_received_at, last_answered_at);
        }
    }

    Ok(Response::builder().body(
        serde_json::to_string(&PostNodeResponse {
            answered_at: Utc::now(),
        })
        .unwrap(),
    ))
}
