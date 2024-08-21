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
        let header_value = req
            .headers()
            .get(header::CONTENT_TYPE)
            .ok_or((StatusCode::BAD_REQUEST, "Content Type not found").into_response())?;

        let content_type = header_value
            .to_str()
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()).into_response())?;

        if content_type != "application/ejson" {
            return Err((StatusCode::BAD_REQUEST, "Content Type not accepted").into_response());
        }

        let body_bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.body_text()).into_response())?;

        let body_ejson: Value = serde_json::from_reader(Cursor::new(body_bytes))
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()).into_response())?;

        let body_bson: Bson = body_ejson
            .try_into()
            .map_err(|_| (StatusCode::BAD_REQUEST, "EJSON to BSON parse error").into_response())?;

        let body_struct: T = T::deserialize(bson::Deserializer::new(body_bson.into()))
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()).into_response())?;

        Ok(EJSON(body_struct))
    }
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
