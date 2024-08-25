use axum::{
    extract::State,
    response::{IntoResponse, Response},
};
use mongodb::{
    self, bson::Document, options::InsertManyOptions, results::InsertManyResult, Client,
};
use serde::Deserialize;

use crate::ejson::EJSON;

#[derive(Debug, Deserialize)]
pub struct InsertManyBody {
    db: String,
    collection: String,
    documents: Vec<Document>,
    options: Option<InsertManyOptions>,
}

pub async fn handler(
    State(client): State<Client>,
    EJSON(args): EJSON<InsertManyBody>,
) -> Result<EJSON<InsertManyResult>, Response> {
    println!("\nArgs: {:#?}", args);

    let result = client
        .database(&args.db)
        .collection(&args.collection)
        .insert_many(args.documents)
        .with_options(args.options)
        .await
        .map_err(|error| EJSON(error).into_response())?;

    println!("\nResult: {:#?}", result);

    Ok(EJSON(result))
}
