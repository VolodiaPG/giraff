extern crate handler;

#[tokio::main]
async fn main() {
    warp::serve(handler::main())
        .run(([127, 0, 0, 1], 3000))
        .await;
}
