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

        // println!("Contetn Type: {:?}", content_type);

        let deserializer = match content_type {
            "application/bson" => req_bytes_to_bson_deserializer(req, state).await.unwrap(),
            _ => return Err((StatusCode::BAD_REQUEST, "Deu ruim").into_response()),
        };

        let find_one_body = T::deserialize(deserializer).unwrap();

        Ok(CustomParser(find_one_body))
    }
}

async fn req_bytes_to_bson_deserializer<S: Send + Sync>(
    req: Request,
    state: &S,
) -> Result<bson::Deserializer, Response> {
    println!("Application/BSON");

    let body = Bytes::from_request(req, state)
        .await
        .map_err(|err| err.into_response())?;

    let reader = Cursor::new(body);
    let teste: Value = serde_json::from_reader(reader).unwrap();
    let filter = Document::from_reader(reader).unwrap();
    let bson_deserializer = bson::Deserializer::new(filter.into());

    Ok(bson_deserializer)
}

pub async fn handler(
    CustomParser(FindOneBody { filter, options }): CustomParser<FindOneBody>,
) -> Bytes {
    println!("Find One Handler");

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
