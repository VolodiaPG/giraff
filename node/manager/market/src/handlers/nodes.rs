use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;
use warp::{http::Response, Rejection};

use crate::live_store::NodesDataBase;
use crate::Error;
use shared_models::node::{PostNode, PostNodeResponse};

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

    if let Some(created_at) = payload.created_at {
        nodes_db
            .lock()
            .await
            .get_mut(&payload.from)
            .ok_or_else(|| warp::reject::custom(Error::NodeIdNotFound(payload.from)))?
            .latency
            .update(Utc::now(), created_at);
    }

    Ok(Response::builder().body(
        serde_json::to_string(&PostNodeResponse {
            answered_at: Utc::now(),
        })
        .unwrap(),
    ))
}
