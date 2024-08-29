use mongodb::Client;

pub async fn get_client() -> Client {
    let uri = "mongodb://127.0.0.1:27018";

    Client::with_uri_str(uri).await.unwrap()
}
