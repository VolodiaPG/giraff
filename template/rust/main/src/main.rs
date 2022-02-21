
use std::error::Error;
use std::io::{self, Read};

extern crate handler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    handle.read_to_string(&mut buffer)?;
    
    let res = handler::handle(buffer).await?;
    println!("{}", res);
    Ok(())
}
