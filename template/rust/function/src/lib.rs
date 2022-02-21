use std::error::Error;

pub async fn handle(req : String) -> Result<String, Box<dyn Error>> {
    Ok(req)
}