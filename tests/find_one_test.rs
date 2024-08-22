use mongodb::{
    self,
    bson::{self, doc, oid::ObjectId, Bson, Document},
    Collection,
};
use reqwest::{
    header::{self},
    Response,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct FindOneBody {
    pub db: String,
    pub collection: String,
    pub filter: Document,
    pub options: Option<Document>,
}

async fn get_mdb_client() -> mongodb::Client {
    mongodb::Client::with_uri_str("mongodb://127.0.0.1:27018")
        .await
        .unwrap()
}

async fn req_post(body: String) -> reqwest::Response {
    reqwest::Client::builder()
        .build()
        .unwrap()
        .post("http://127.0.0.1:8080/findOne")
        .header(header::CONTENT_TYPE, "application/ejson")
        .body(body)
        .send()
        .await
        .unwrap()
}

async fn serialize_find_one_body(find_one_body: FindOneBody) -> String {
    find_one_body
        .serialize(bson::Serializer::new())
        .unwrap()
        .into_canonical_extjson()
        .to_string()
}

async fn deserialize_find_one_result(res: Response) -> Option<Document> {
    let body_ejson: Value = res.json().await.unwrap();
    let body_bson: Bson = body_ejson.try_into().unwrap();

    body_bson.as_document().map(|doc| doc.clone())
}

#[tokio::test]
async fn find_one() {
    // Data Setup
    let db = format!("test-{}", ObjectId::new().to_string());
    let collection = "users".to_string();
    let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };

    // MDB Setup
    let mdb_client = get_mdb_client().await;
    let mdb_db = mdb_client.database(&db);
    let mdb_collection: Collection<Document> = mdb_db.collection(&collection);

    mdb_collection.delete_many(doc! {}).await.unwrap();
    mdb_collection.insert_one(&user).await.unwrap();

    // Request
    let find_one_body = FindOneBody {
        db,
        collection,
        filter: doc! {"name": "john"},
        options: None,
    };
    let body = serialize_find_one_body(find_one_body).await;
    let res = req_post(body).await;
    let document = deserialize_find_one_result(res).await.unwrap();

    assert_eq!(document, user);

    mdb_db.drop().await.unwrap();
}

#[tokio::test]
async fn find_one_with_options() {
    // Data Setup
    let db = format!("test-{}", ObjectId::new().to_string());
    let collection = "users".to_string();
    let user_2 = doc! { "_id": ObjectId::new(), "name": "ane", "age": 30 };
    let user_1 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };

    // MDB Setup
    let mdb_client = get_mdb_client().await;
    let mdb_db = mdb_client.database(&db);
    let mdb_collection: Collection<Document> = mdb_db.collection(&collection);

    mdb_collection.delete_many(doc! {}).await.unwrap();
    mdb_collection
        .insert_many([&user_1, &user_2])
        .await
        .unwrap();

    // Request
    let find_one_body = FindOneBody {
        db,
        collection,
        filter: doc! {"age": 30},
        options: Some(doc! {
            "projection": doc! {"_id": 0, "name": 1},
            "sort": doc! {"name": 1}
        }),
    };
    let body = serialize_find_one_body(find_one_body).await;
    let res = req_post(body).await;
    let document = deserialize_find_one_result(res).await.unwrap();

    assert_eq!(document, doc! {"name": "ane"});

    mdb_db.drop().await.unwrap();
}
