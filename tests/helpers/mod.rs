#![allow(unused)]

use axum::{
    body::{to_bytes, Body},
    http::{header, response::Parts, Method, Request},
};
use mongodb::{
    bson::{self, doc, oid::ObjectId, Array, Bson, Document},
    Collection, Database,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tower::ServiceExt;

use rs_data_api::{app, mdb};

pub async fn get_db_and_collection() -> (Database, Collection<Document>) {
    let client = mdb::get_client().await;
    let db = client.database(&format!("test-{}", ObjectId::new().to_string()));
    let collection = db.collection::<Document>("documents");

    collection.delete_many(doc! {}).await.unwrap();

    (db, collection)
}

pub fn get_body_ejson_from_struct(structure: impl Serialize) -> Body {
    let structure_json = structure
        .serialize(bson::Serializer::new())
        .unwrap()
        .into_canonical_extjson();

    Body::from(serde_json::to_vec(&structure_json).unwrap())
}

pub async fn get_document_from_body(body: Body) -> Document {
    let body_bytes = to_bytes(body, usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let body_bson: Bson = body_json.try_into().unwrap();

    body_bson.as_document().unwrap().clone()
}

pub async fn get_array_from_body(body: Body) -> Array {
    let body_bytes = to_bytes(body, usize::MAX).await.unwrap();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    let body_bson: Bson = body_json.try_into().unwrap();

    body_bson.as_array().unwrap().clone()
}

pub fn get_struct_from_doc<T: DeserializeOwned>(doc: Document) -> T {
    T::deserialize(bson::Deserializer::new(bson::from_document(doc).unwrap())).unwrap()
}

pub fn build_request(uri: &str, body: Body) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/ejson")
        .body(body)
        .unwrap()
}

pub async fn one_shot(uri: &str, body: impl Serialize) -> (Parts, Body) {
    let body_ejson = get_body_ejson_from_struct(body);
    let request = build_request(uri, body_ejson);
    let app_router = app::build().await;
    let (parts, body) = app_router.oneshot(request).await.unwrap().into_parts();

    (parts, body)
}

pub async fn one_shot_document(uri: &str, body: impl Serialize) -> (Parts, Document) {
    let (parts, body) = one_shot(uri, body).await;
    let doc = get_document_from_body(body).await;

    (parts, doc)
}

pub async fn one_shot_array(uri: &str, body: impl Serialize) -> (Parts, Vec<Bson>) {
    let (parts, body) = one_shot(uri, body).await;
    let doc = get_array_from_body(body).await;

    (parts, doc)
}
