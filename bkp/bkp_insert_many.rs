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

//
pub async fn get_mdb_client() -> mongodb::Client {
    mongodb::Client::with_uri_str("mongodb://127.0.0.1:27018")
        .await
        .unwrap()
}

//

#[derive(Debug, Deserialize, Serialize)]
pub struct InsertManyBody {
    db: String,
    collection: String,
    documents: Vec<Document>,
    options: Option<Document>,
}

// async fn get_mdb_client() -> mongodb::Client {
//     mongodb::Client::with_uri_str("mongodb://127.0.0.1:27018")
//         .await
//         .unwrap()
// }

async fn req_post(body: String) -> reqwest::Response {
    reqwest::Client::builder()
        .build()
        .unwrap()
        .post("http://127.0.0.1:8080/insertMany")
        .header(header::CONTENT_TYPE, "application/ejson")
        .body(body)
        .send()
        .await
        .unwrap()
}

async fn serialize_insert_many_body(insert_many_body: InsertManyBody) -> String {
    insert_many_body
        .serialize(bson::Serializer::new())
        .unwrap()
        .into_canonical_extjson()
        .to_string()
}

async fn deserialize_insert_many_result(res: Response) -> Option<Document> {
    let body_ejson: Value = res.json().await.unwrap();
    let body_bson: Bson = body_ejson.try_into().unwrap();

    body_bson.as_document().map(|doc| doc.clone())
}

// #[tokio::test]
// async fn insert_one() {
//     // Data Setup
//     let db = format!("test-{}", ObjectId::new().to_string());
//     let collection = "users".to_string();
//     let user_0 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
//     let user_1 = doc! { "_id": ObjectId::new(), "name": "jim", "age": 30 };

//     // MDB Setup
//     let mdb_client = get_mdb_client().await;
//     let mdb_db = mdb_client.database(&db);
//     let mdb_collection: Collection<Document> = mdb_db.collection(&collection);

//     mdb_collection.delete_many(doc! {}).await.unwrap();

//     // Request
//     let insert_many_body = InsertManyBody {
//         db,
//         collection,
//         documents: vec![user_0.clone(), user_1.clone()],
//         options: None,
//     };
//     let body = serialize_insert_many_body(insert_many_body).await;
//     let res = req_post(body).await;
//     let result = deserialize_insert_many_result(res).await.unwrap();

//     assert_eq!(
//         result.get_document("inserted_ids").unwrap().get("0"),
//         user_0.get("_id")
//     );

//     assert_eq!(
//         result.get_document("inserted_ids").unwrap().get("1"),
//         user_1.get("_id")
//     );

//     mdb_db.drop().await.unwrap();
// }

#[tokio::test]
async fn insert_one_error() {
    // Data Setup
    let db = format!("test-{}", ObjectId::new().to_string());
    let collection = "users".to_string();
    let user_0 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
    let user_1 = doc! { "_id": ObjectId::new(), "name": "jim", "age": 30 };

    // MDB Setup
    let mdb_client = get_mdb_client().await;
    let mdb_db = mdb_client.database(&db);
    let mdb_collection: Collection<Document> = mdb_db.collection(&collection);

    mdb_collection.delete_many(doc! {}).await.unwrap();
    mdb_collection.insert_one(&user_1).await.unwrap();

    // Request
    let insert_many_body = InsertManyBody {
        db,
        collection,
        documents: vec![user_0.clone(), user_1.clone()],
        options: Some(doc! {
            "ordered": false,
        }),
    };
    let body = serialize_insert_many_body(insert_many_body).await;
    let res = req_post(body).await;
    let result = deserialize_insert_many_result(res).await.unwrap();

    println!("Result: {:#?}", result);

    mdb_db.drop().await.unwrap();
}

// #[tokio::test]
// async fn insert_one_with_options() {
//     // Data Setup
//     let db = format!("test-{}", ObjectId::new().to_string());
//     let collection = "users".to_string();
//     let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };

//     // MDB Setup
//     let mdb_client = get_mdb_client().await;
//     let mdb_db = mdb_client.database(&db);
//     let mdb_collection: Collection<Document> = mdb_db.collection(&collection);

//     mdb_collection.delete_many(doc! {}).await.unwrap();

//     // Request
//     let insert_many_body = InsertManyBody {
//         db,
//         collection,
//         document: user.clone(),
//         options: Some(doc! {
//             "bypassDocumentValidation": true,
//             "comment": "My insert one operation",
//             "writeConcern": doc! {
//                 "j": false,
//                 "w": "majority",
//                 "wtimeout": 200,
//             }
//         }),
//     };
//     let body = serialize_insert_many_body(insert_many_body).await;
//     let res = req_post(body).await;
//     let result = deserialize_insert_many_result(res).await.unwrap();

//     assert_eq!(result.get("inserted_id"), user.get("_id"));

//     mdb_db.drop().await.unwrap();
// }
