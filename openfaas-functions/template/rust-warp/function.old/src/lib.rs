use serde::{Deserialize, Serialize};
use warp::{filters::BoxedFilter, Filter, Reply};

/// # Response Struct
///
/// `Response` implements `warp::Reply` and is serializable into JSON.
/// The purpose of the Struct is to act as a container for the repsonse body.
///
/// It takes a user agent as a parameter and uses it to define a private `message` property.
///
/// `Response`'s implementation of `warp::Reply` allows `Response` to be called in a `warp` filter's `map` method.
///
/// `Response` will be serialized into JSON string and sent to the client.
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    message: String,
}

impl warp::Reply for Response {
    fn into_response(self) -> warp::reply::Response {
        warp::reply::json(&self).into_response()
    }
}

impl Response {
    fn new(user_agent: String) -> Response {
        Response {
            message: create_message(&user_agent),
        }
    }
}

fn create_message(user_agent: &str) -> String {
    format!("Hello world!!!! Your user agent is {}", user_agent)
}

/// # Usage in function wrapper
///
/// `main` returns a `BoxedFilter` that is consumed by the [rust-warp template](https://github.com/austinrivas/rust-warp-template).
///
/// ```
/// warp::serve(handler::main())
///     .run(([127, 0, 0, 1], 3000));
/// ```
pub fn main() -> BoxedFilter<(impl Reply,)> {
    warp::get()
        .and(warp::header("user-agent"))
        .map(Response::new)
        .boxed()
}
