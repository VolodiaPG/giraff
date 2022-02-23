use hyper::service::{make_service_fn, service_fn};
use hyper::{Request, Response, StatusCode, Server, Body};
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use tokio;

extern crate handler;

fn finalize(result: Result<Response<Body>, Box<dyn Error>>) -> Response<Body> {
    match result {
        Ok(response) => response,
        Err(error) => {
            let body = format!(
                "{{\"status\": \"{}\", \"description\":\"{}\"}}",
                StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                error.to_string()
            );
            let mut resp = Response::new(Body::from(body));
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            resp
        }
    }
}

async fn handler_service(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let resp = finalize(handler::handle(req).await);
    Ok(resp)
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handler_service)) });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
