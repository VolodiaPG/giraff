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
    let client = redis::Client::open("redis://redis-server/")?;
    let mut con = client.get_connection()?;
   
    let light: String = con.get("light")?;
    let blink: String = con.get("blink")?;

    Ok(format!("light:{},blink: {}", light, blink))
}
