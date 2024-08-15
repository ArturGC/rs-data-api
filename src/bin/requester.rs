use axum::http::Request;

#[tokio::main]
async fn main() {
    let request = Request::builder()
        .method("GET")
        .uri("https://www.rust-lang.org/")
        .header("X-Custom-Foo", "Bar")
        .body(())
        .unwrap();
}
