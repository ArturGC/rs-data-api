use crate::ejson::EJSON;
use axum::extract::State;
use mongodb::{bson::Document, options::FindOneOptions, Client};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FindOneBody {
    db: String,
    collection: String,
    filter: Document,
    options: Option<FindOneOptions>,
}

pub async fn handler(
    State(client): State<Client>,
    EJSON(args): EJSON<FindOneBody>,
) -> Result<EJSON<Option<Document>>, EJSON<mongodb::error::Error>> {
    let result = client
        .database(&args.db)
        .collection(&args.collection)
        .find_one(args.filter)
        .with_options(args.options)
        .await
        .map_err(|error| EJSON(error))?;

    Ok(EJSON(result))
}
