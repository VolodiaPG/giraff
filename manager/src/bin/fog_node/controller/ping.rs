use manager::model::view::ping::{Ping, PingResponse};

pub async fn ping(_: Ping) -> PingResponse { PingResponse { received_at: chrono::Utc::now() } }
