use crate::ejson::EJSON;
use axum::extract::State;
use futures::stream::TryStreamExt;
use mongodb::{bson::Document, options::FindOptions, Client};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FindBody {
    db: String,
    collection: String,
    filter: Document,
    options: Option<FindOptions>,
}

pub async fn handler(
    State(client): State<Client>,
    EJSON(args): EJSON<FindBody>,
) -> Result<EJSON<Vec<Document>>, EJSON<mongodb::error::Error>> {
    let cursor = client
        .database(&args.db)
        .collection::<Document>(&args.collection)
        .find(args.filter)
        .with_options(args.options)
        .await
        .map_err(|error| EJSON(error))?;

    let result: Vec<Document> = cursor.try_collect().await.map_err(|error| EJSON(error))?;

    Ok(EJSON(result))
}
