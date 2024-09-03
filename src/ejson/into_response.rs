use axum::{
    body::Body,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use mongodb::{
    self,
    bson::{self, Bson, Document},
    error::ErrorKind,
    results::{InsertManyResult, InsertOneResult, UpdateResult},
};
use serde::Serialize;
use serde_json::json;

use super::EJSON;

fn struct_to_ejson_string(structure: impl Serialize) -> String {
    structure
        .serialize(bson::Serializer::new())
        .unwrap()
        .into_canonical_extjson()
        .to_string()
}

impl IntoResponse for EJSON<Option<Document>> {
    fn into_response(self) -> Response {
        let body_bson: Bson = self.0.into();
        let body_ejson_string = body_bson.into_canonical_extjson().to_string();

        Response::builder()
            .status(StatusCode::ACCEPTED)
            .header(header::CONTENT_TYPE, "application/ejson")
            .body(Body::from(body_ejson_string))
            .unwrap()
    }
}

impl IntoResponse for EJSON<Vec<Document>> {
    fn into_response(self) -> Response {
        let body_bson: Bson = self.0.into();
        let body_ejson_string = body_bson.into_canonical_extjson().to_string();

        Response::builder()
            .status(StatusCode::ACCEPTED)
            .header(header::CONTENT_TYPE, "application/ejson")
            .body(Body::from(body_ejson_string))
            .unwrap()
    }
}

impl IntoResponse for EJSON<InsertOneResult> {
    fn into_response(self) -> Response {
        let body_ejson_string = struct_to_ejson_string(self.0);

        Response::builder()
            .status(StatusCode::ACCEPTED)
            .header(header::CONTENT_TYPE, "application/ejson")
            .body(Body::from(body_ejson_string))
            .unwrap()
    }
}

impl IntoResponse for EJSON<InsertManyResult> {
    fn into_response(self) -> Response {
        let body_ejson_string = struct_to_ejson_string(self.0);

        Response::builder()
            .status(StatusCode::ACCEPTED)
            .header(header::CONTENT_TYPE, "application/ejson")
            .body(Body::from(body_ejson_string))
            .unwrap()
    }
}

impl IntoResponse for EJSON<UpdateResult> {
    fn into_response(self) -> Response {
        let body_ejson_string = struct_to_ejson_string(self.0);

        Response::builder()
            .status(StatusCode::ACCEPTED)
            .header(header::CONTENT_TYPE, "application/ejson")
            .body(Body::from(body_ejson_string))
            .unwrap()
    }
}

impl IntoResponse for EJSON<mongodb::error::Error> {
    fn into_response(self) -> Response {
        let data = match *self.0.kind {
            ErrorKind::InsertMany(e) => struct_to_ejson_string(e),
            ErrorKind::Write(e) => struct_to_ejson_string(e),
            ErrorKind::Command(e) => struct_to_ejson_string(e),
            e => json!({"message": e.to_string()}).to_string(),
        };

        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(header::CONTENT_TYPE, "application/ejson")
            .body(Body::from(data))
            .unwrap()
    }
}
