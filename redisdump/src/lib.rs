use redis::Commands;
use std::error::Error;
use std::result::Result;

// {
//     "temperature_celsius": 25.4,
//     "humidity_percent": 70.0,
//     "wind_kph": 100.0,
//     "rain": false
//     }

pub async fn handle(_req: String) -> Result<String, Box<dyn Error>> {
    let client = redis::Client::open("redis://redis-server.redis-server.svc.cluster.local:6379/")?;
    let mut con = client.get_connection()?;
   
    let ret: String = con.get("foo")?;
    Ok(ret)
}
