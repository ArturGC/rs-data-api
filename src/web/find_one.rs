use std::io::Cursor;

use axum::{
    async_trait,
    body::Bytes,
    extract::{rejection::BytesRejection, FromRequest, FromRequestParts, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use mongodb::{
    bson::{self, Bson, Document},
    options::FindOneOptions,
    Client, Collection,
};
use serde::{de::DeserializeOwned, Deserialize, Deserializer};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct FindOneBody {
    filter: Document,
    options: FindOneOptions,
}

pub struct CustomParser<T>(T);

#[async_trait]
impl<T, S> FromRequest<S> for CustomParser<T>
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

        let body_bytes = Bytes::from_request(req, state).await.unwrap();
        let body_ejson: Value = serde_json::from_reader(Cursor::new(body_bytes)).unwrap();
        let body_bson: Bson = body_ejson.try_into().unwrap();
        let body_struct = T::deserialize(bson::Deserializer::new(body_bson.into())).unwrap();

        Ok(CustomParser(body_struct))
    }
}

pub async fn handler(
    CustomParser(FindOneBody { filter, options }): CustomParser<FindOneBody>,
) -> Bytes {
    println!("Find One Handler");
    println!("Filter: {:?}", filter);
    println!("Options: {:?}", options);

    let collection = get_users_collection().await;
    let result = collection
        .find_one(filter)
        .with_options(options)
        .await
        .unwrap()
        .unwrap();

    println!("\nDocument: {:?}", result);

    let mut vector: Vec<u8> = Vec::new();

    result.to_writer(&mut vector).unwrap();

    vector.into()
}

async fn get_users_collection() -> Collection<Document> {
    let uri = "mongodb://127.0.0.1:27018";
    let client = Client::with_uri_str(uri).await.unwrap();

    client.database("test").collection("users")
}
