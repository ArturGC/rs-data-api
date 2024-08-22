use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::{bson::Document, options::FindOneOptions, Client};
use serde::Deserialize;

use crate::web::ejson::EJSON;

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
) -> Result<EJSON<Option<Document>>, Response> {
    let result = client
        .database(&args.db)
        .collection(&args.collection)
        .find_one(args.filter)
        .with_options(args.options)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response())?;

    Ok(EJSON(result))
}
