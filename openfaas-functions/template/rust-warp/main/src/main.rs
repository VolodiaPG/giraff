extern crate handler;

use std::env;

#[tokio::main]
async fn main() {
    let debug = !env::var("DEBUG").is_err();

    let app = warp::serve(handler::main());
    if debug{
        app.run(([0, 0, 0, 0], 3000)).await;
    }
    else{
        app.run(([127, 0, 0, 1], 3000)).await;
    }
}
