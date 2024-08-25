use rs_data_api::app;

#[tokio::main]
async fn main() {
    let app_router = app::build().await;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    axum::serve(listener, app_router).await.unwrap();
}
