use redis::Commands;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::result::Result;
use validator::Validate;

// {
//     "temperature_celsius": 25.4,
//     "humidity_percent": 70.0,
//     "wind_kph": 100.0,
//     "rain": false
//     }

#[derive(Deserialize, Validate)]
struct IncomingPayload {
    road_condition: u8,
}

pub async fn handle(req: String) -> Result<String, Box<dyn Error>> {
    let des: IncomingPayload = serde_json::from_str(req.as_str())?;
    des.validate()?;
   
    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;
   
    con.set("foo", des.road_condition)?;
    
    Ok("".to_string())
}
