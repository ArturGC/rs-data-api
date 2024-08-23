use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::{self, bson::Document, options::InsertOneOptions, results::InsertOneResult, Client};
use serde::Deserialize;
use serde_json::json;

use crate::web::ejson::EJSON;

#[derive(Debug, Deserialize)]
pub struct InsertOneBody {
    db: String,
    collection: String,
    document: Document,
    options: Option<InsertOneOptions>,
}

pub async fn handler(
    State(client): State<Client>,
    EJSON(args): EJSON<InsertOneBody>,
) -> Result<EJSON<InsertOneResult>, Response> {
    let result = client
        .database(&args.db)
        .collection(&args.collection)
        .insert_one(args.document)
        .with_options(args.options)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({"message": e.to_string()}).to_string(),
            )
                .into_response()
        })?;

    Ok(EJSON(result))
}
