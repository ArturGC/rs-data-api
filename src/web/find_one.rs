use mongodb::{bson::Document, options::FindOneOptions, Client, Collection};
use serde::Deserialize;

use crate::web::ejson::EJSON;

#[derive(Debug, Deserialize)]
pub struct FindOneBody {
    filter: Document,
    options: Option<FindOneOptions>,
}

pub async fn handler(EJSON(args): EJSON<FindOneBody>) -> EJSON<Option<Document>> {
    println!("\nFind One Handler");
    println!("\nFilter: {:?}", args.filter);
    println!("\nOptions: {:?}", args.options);

    let collection = get_users_collection().await;
    let result = collection
        .find_one(args.filter)
        .with_options(args.options)
        .await
        .unwrap();

    println!("\nDocument: {:?}", result);

    EJSON(result)
}

async fn get_users_collection() -> Collection<Document> {
    let uri = "mongodb://127.0.0.1:27018";
    let client = Client::with_uri_str(uri).await.unwrap();

    client.database("test").collection("users")
}
