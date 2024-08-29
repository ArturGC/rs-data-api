use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use mongodb::{
    self,
    bson::{self, bson, Bson, Document},
    error::ErrorKind,
    results::{InsertManyResult, InsertOneResult},
};
use serde::Serialize;
use serde_json::json;

use super::EJSON;

fn bson_to_ejson_string(something: impl Serialize) -> String {
    something
        .serialize(bson::Serializer::new())
        .unwrap()
        .into_canonical_extjson()
        .to_string()
}

impl IntoResponse for EJSON<Option<Document>> {
    fn into_response(self) -> Response {
        let body_bson: Bson = self.0.into();
        let body_ejson = body_bson.into_canonical_extjson();

        (
            StatusCode::ACCEPTED,
            [(header::CONTENT_TYPE, "application/ejson")],
            body_ejson.to_string(),
        )
            .into_response()
    }
}

impl IntoResponse for EJSON<InsertOneResult> {
    fn into_response(self) -> Response {
        let body_bson: Bson = bson!({ "inserted_id": self.0.inserted_id });
        let body_ejson = body_bson.into_canonical_extjson();

        (
            StatusCode::ACCEPTED,
            [(header::CONTENT_TYPE, "application/ejson")],
            body_ejson.to_string(),
        )
            .into_response()
    }
}

impl IntoResponse for EJSON<InsertManyResult> {
    fn into_response(self) -> Response {
        let original_iter = self.0.inserted_ids.into_iter();
        let formatted_iter = original_iter.map(|(i, id)| (i.to_string(), id));
        let body_bson: Bson = bson!({ "inserted_ids": Document::from_iter(formatted_iter) });
        let body_ejson = body_bson.into_canonical_extjson();

        (
            StatusCode::ACCEPTED,
            [(header::CONTENT_TYPE, "application/ejson")],
            body_ejson.to_string(),
        )
            .into_response()
    }
}

impl IntoResponse for EJSON<mongodb::error::Error> {
    fn into_response(self) -> Response {
        let body = match *self.0.kind {
            ErrorKind::InsertMany(e) => bson_to_ejson_string(e),
            ErrorKind::Write(e) => bson_to_ejson_string(e),
            ErrorKind::Command(e) => bson_to_ejson_string(e),
            e => json!({"message": e.to_string()}).to_string(),
        };

        (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "application/ejson")],
            body,
        )
            .into_response()
    }
}
