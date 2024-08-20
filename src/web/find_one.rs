use axum::body::Bytes;
use mongodb::{
    bson::{self, Bson, Document},
    options::FindOneOptions,
    Client, Collection,
};
use serde::Deserialize;

use super::custom_parser::Parser;

#[derive(Debug, Deserialize)]
pub struct FindOneBody {
    filter: Document,
    options: Option<FindOneOptions>,
}

pub async fn handler(Parser(args): Parser<FindOneBody>) -> String {
    println!("\nFind One Handler");
    println!("\nFilter: {:#?}", args.filter);
    println!("\nOptions: {:#?}", args.options);

    let collection = get_users_collection().await;
    let result = collection
        .find_one(args.filter)
        .with_options(args.options)
        .await
        .unwrap()
        .unwrap();

    println!("\nDocument: {:?}", result);

    let doc_bson: Bson = result.into();
    let doc_ejson = doc_bson.into_canonical_extjson();
    let doc_string = doc_ejson.to_string();

    doc_string
}

async fn get_users_collection() -> Collection<Document> {
    let uri = "mongodb://127.0.0.1:27018";
    let client = Client::with_uri_str(uri).await.unwrap();

    client.database("test").collection("users")
}
