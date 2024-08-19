use axum::body::Bytes;
use mongodb::{bson::Document, options::FindOneOptions, Client, Collection};
use serde::Deserialize;

use super::custom_parser::CustomParser;

#[derive(Debug, Deserialize)]
pub struct FindOneBody {
    filter: Document,
    options: Option<FindOneOptions>,
}

pub async fn handler(
    CustomParser(FindOneBody { filter, options }): CustomParser<FindOneBody>,
) -> Bytes {
    println!("\nFind One Handler");
    println!("\nFilter: {:#?}", filter);
    println!("\nOptions: {:#?}", options);

    let collection = get_users_collection().await;
    let result = collection
        .find_one(filter)
        .with_options(options)
        .await
        .unwrap()
        .unwrap();

    println!("\nDocument: {:?}", result);

    let mut vector: Vec<u8> = Vec::new();

    result.to_writer(&mut vector).unwrap();

    vector.into()
}

async fn get_users_collection() -> Collection<Document> {
    let uri = "mongodb://127.0.0.1:27018";
    let client = Client::with_uri_str(uri).await.unwrap();

    client.database("test").collection("users")
}
