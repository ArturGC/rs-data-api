use axum::{
    async_trait,
    body::{Body, Bytes},
    extract::{FromRequest, Request},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson::{self, Bson};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::ops::Deref;

use super::EJSON;

#[async_trait]
impl<T, S> FromRequest<S> for EJSON<T>
where
    S: Send + Sync,
    T: DeserializeOwned,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let header_value = get_header_value(&req)?;
        let content_type = get_content_type(header_value)?;

        validate_content_type(content_type)?;

        let body_bytes = get_body_as_bytes(req, state).await?;
        let body_json = bytes_to_json(body_bytes)?;
        let body_bson = json_to_bson(body_json)?;
        let body_struct = bson_to_struct::<T>(body_bson)?;

        Ok(EJSON(body_struct))
    }
}

fn get_header_value<'a>(req: &'a Request) -> Result<&'a HeaderValue, Response<Body>> {
    let header_value = req.headers().get(header::CONTENT_TYPE).ok_or(
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "Content Type not found"})),
        )
            .into_response(),
    )?;

    Ok(header_value)
}

fn get_content_type<'a>(header_value: &'a HeaderValue) -> Result<&'a str, Response<Body>> {
    let content_type = header_value.to_str().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": e.to_string()})),
        )
            .into_response()
    })?;

    Ok(content_type)
}

fn validate_content_type(content_type: &str) -> Result<(), Response<Body>> {
    if content_type == "application/ejson" {
        return Ok(());
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(json!({"message": "Content Type not accepted"})),
    )
        .into_response())
}

async fn get_body_as_bytes<S: Send + Sync>(
    req: Request,
    state: &S,
) -> Result<Bytes, Response<Body>> {
    let body_bytes = Bytes::from_request(req, state).await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": e.body_text()})),
        )
            .into_response()
    })?;

    Ok(body_bytes)
}

fn bytes_to_json(bytes: Bytes) -> Result<Value, Response<Body>> {
    let body_ejson: Value = serde_json::from_slice(bytes.deref()).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": e.to_string()})),
        )
            .into_response()
    })?;

    Ok(body_ejson)
}

fn json_to_bson(value: Value) -> Result<Bson, Response<Body>> {
    let body_bson: Bson = value.try_into().map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": "JSON to BSON parse error"})),
        )
            .into_response()
    })?;

    Ok(body_bson)
}

fn bson_to_struct<T: DeserializeOwned>(bson: Bson) -> Result<T, Response<Body>> {
    let body_struct: T = T::deserialize(bson::Deserializer::new(bson)).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"message": e.to_string()})),
        )
            .into_response()
    })?;

    Ok(body_struct)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body, Bytes},
        extract::Request,
        http::{HeaderValue, StatusCode},
        Json,
    };

    async fn body_to_json(body: Body) -> Json<Value> {
        let body_bytes = to_bytes(body, usize::MAX).await.unwrap();
        let body_json: Json<Value> = Json::from_bytes(&*body_bytes).unwrap();

        body_json
    }

    #[tokio::test]
    async fn get_header_value_no_header() {
        let req = Request::new(Body::new::<String>(String::new()));
        let res = get_header_value(&req).err().unwrap();
        let (parts, body) = res.into_parts();
        let body_json = body_to_json(body).await;
        let message = body_json.get("message").unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "Content Type not found");
    }

    #[tokio::test]
    async fn get_content_type_not_str() {
        let header_value = HeaderValue::from_str("🤷‍♂️").unwrap();
        let res = get_content_type(&header_value).err().unwrap();
        let (parts, body) = res.into_parts();
        let body_json = body_to_json(body).await;
        let message = body_json.get("message").unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "failed to convert header to a str");
    }

    #[tokio::test]
    async fn validate_content_type_not_ejson() {
        let content_type = "not_ejson";
        let res = validate_content_type(content_type).err().unwrap();
        let (parts, body) = res.into_parts();
        let body_json = body_to_json(body).await;
        let message = body_json.get("message").unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "Content Type not accepted");
    }

    #[tokio::test]
    async fn bytes_to_json_not_json() {
        let bytes = Bytes::from("{ name: john }");
        let res = bytes_to_json(bytes).err().unwrap();
        let (parts, body) = res.into_parts();
        let body_json = body_to_json(body).await;
        let message = body_json.get("message").unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "key must be a string at line 1 column 3");
    }

    #[tokio::test]
    async fn json_to_bson_not_bson() {
        let value: Json<Value> = Json(json!({ "$numberLong": 5}));
        let res = json_to_bson(value.0).err().unwrap();
        let (parts, body) = res.into_parts();
        let body_json = body_to_json(body).await;
        let message = body_json.get("message").unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "JSON to BSON parse error");
    }

    #[tokio::test]
    async fn bson_to_struct_wrong_bson() {
        use mongodb::bson::bson;
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct User {
            #[allow(dead_code)]
            name: String,
        }

        let bson = bson!({"message": "hello world"});
        let res = bson_to_struct::<User>(bson).err().unwrap();
        let (parts, body) = res.into_parts();
        let body_json = body_to_json(body).await;
        let message = body_json.get("message").unwrap();

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert_eq!(message, "missing field `name`");
    }
}
