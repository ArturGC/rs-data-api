use std::io::Cursor;

use mongodb::bson::{self, doc, Bson, Document};
use reqwest::header::{self, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct FindOneOptions {
    pub comment: Option<Bson>,
    pub projection: Option<Document>,
    pub skip: Option<u64>,
    pub sort: Option<Document>,
}

#[derive(Serialize, Deserialize)]
struct FindOneBody {
    pub filter: Document,
    pub options: Option<Document>,
}

#[tokio::test]
async fn find_one_path() {
    let body = FindOneBody {
        filter: doc! {"name": "artur"},
        options: Some(doc! {
            "projection": doc! {"_id": 0},
        }),
    };

    let bson_serializer = bson::Serializer::new();
    let body_bson = body.serialize(bson_serializer).unwrap();
    let req_body = body_bson.into_canonical_extjson().to_string();

    println!("Request Body: {:?}", req_body);

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/ejson".parse().unwrap());

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .unwrap();

    let res = client
        .post("http://127.0.0.1:8080/findOne")
        .body(req_body)
        .send()
        .await
        .unwrap();

    let content_type = res.headers().get(CONTENT_TYPE).unwrap();
    println!("Content Type: {:?}", content_type);

    let res_body = res.bytes().await.unwrap();
    let reader = Cursor::new(res_body.clone());
    let document = Document::from_reader(reader).unwrap();
    println!("body = {document:?}");
}
