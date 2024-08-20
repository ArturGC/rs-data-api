use std::io::Cursor;

use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Request},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use mongodb::bson::{self, Bson, Document};
use serde::de::DeserializeOwned;
use serde_json::Value;

pub struct EJSON<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for EJSON<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let content_type = req.headers().get("content-type").unwrap().to_str().unwrap();

        if content_type != "application/ejson" {
            return Err((StatusCode::BAD_REQUEST, "Deu ruim").into_response());
        }

        let body_bytes: Bytes = Bytes::from_request(req, state).await.unwrap();
        let body_ejson: Value = serde_json::from_reader(Cursor::new(body_bytes)).unwrap();
        let body_bson: Bson = body_ejson.try_into().unwrap();
        let body_struct: T = T::deserialize(bson::Deserializer::new(body_bson.into())).unwrap();

        Ok(EJSON(body_struct))
    }
}

impl IntoResponse for EJSON<Option<Document>> {
    fn into_response(self) -> Response {
        let EJSON(body) = self;
        let body_bson: Bson = body.into();
        let body_ejson = body_bson.into_canonical_extjson();

        (
            StatusCode::ACCEPTED,
            [(header::CONTENT_TYPE, "application/ejson")],
            body_ejson.to_string(),
        )
            .into_response()
    }
}
