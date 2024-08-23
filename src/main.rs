pub mod web;

use axum::{routing::post, Router};

use mongodb::Client;
use tokio::net::TcpListener;
use web::{find_one, insert_one};

#[tokio::main]
async fn main() {
    let client = get_client().await;
    let listener = get_listener().await;

    let router = Router::new()
        .route("/findOne", post(find_one::handler))
        .route("/insertOne", post(insert_one::handler))
        .with_state(client);

    axum::serve(listener, router).await.unwrap();
}

async fn get_client() -> Client {
    let uri = "mongodb://127.0.0.1:27018";

    Client::with_uri_str(uri).await.unwrap()
}

async fn get_listener() -> TcpListener {
    let addr = "127.0.0.1:8080";

    tokio::net::TcpListener::bind(addr).await.unwrap()
}

// https://github.com/tokio-rs/axum/discussions/1131
// https://stackoverflow.com/questions/76191856/get-access-to-the-request-headers-from-axum-intoresponse
