use crate::ejson::EJSON;
use axum::extract::State;
use mongodb::{bson::Document, options::DeleteOptions, results::DeleteResult, Client};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FindBody {
    db: String,
    collection: String,
    filter: Document,
    options: Option<DeleteOptions>,
}

pub async fn handler(
    State(client): State<Client>,
    EJSON(args): EJSON<FindBody>,
) -> Result<EJSON<DeleteResult>, EJSON<mongodb::error::Error>> {
    let result = client
        .database(&args.db)
        .collection::<Document>(&args.collection)
        .delete_many(args.filter)
        .with_options(args.options)
        .await
        .map_err(|error| EJSON(error))?;

    Ok(EJSON(result))
}
