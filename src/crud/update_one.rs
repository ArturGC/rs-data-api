use crate::ejson::EJSON;
use axum::extract::State;
use mongodb::{bson::Document, options::UpdateOptions, results::UpdateResult, Client};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateOneBody {
    db: String,
    collection: String,
    query: Document,
    update: Document,
    options: Option<UpdateOptions>,
}

pub async fn handler(
    State(client): State<Client>,
    EJSON(args): EJSON<UpdateOneBody>,
) -> Result<EJSON<UpdateResult>, EJSON<mongodb::error::Error>> {
    let result = client
        .database(&args.db)
        .collection::<Document>(&args.collection)
        .update_one(args.query, args.update)
        .with_options(args.options)
        .await
        .map_err(|error| EJSON(error))?;

    Ok(EJSON(result))
}
