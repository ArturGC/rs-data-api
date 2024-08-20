use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    middleware::{from_fn, Next},
    response::Response,
    routing::{get, post},
    Router,
};

mod web;

use web::find_one;

#[tokio::main]
async fn main() {
    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/findOne", post(find_one::handler))
        .route_layer(from_fn(auth));

    axum::serve(listener, router).await.unwrap();
}

async fn auth(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let accept_header = req
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|value| value.to_str().unwrap().to_owned())
        .unwrap();

    println!("Content Type: {accept_header}");

    Ok(next.run(req).await)
}

// https://github.com/tokio-rs/axum/discussions/1131
// https://stackoverflow.com/questions/76191856/get-access-to-the-request-headers-from-axum-intoresponse
