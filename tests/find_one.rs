#[cfg(test)]
mod tests {
    use axum::{
        body::{to_bytes, Body},
        http::{header, Method, Request, StatusCode},
    };
    use mongodb::{
        bson::{self, doc, oid::ObjectId, Bson, Document},
        Collection, Database,
    };
    use rs_data_api::{app, mdb};
    use serde::{Deserialize, Serialize};
    use serde_json::Value;
    use tower::ServiceExt;

    #[derive(Serialize, Deserialize)]
    struct FindOneBody {
        pub db: String,
        pub collection: String,
        pub filter: Document,
        pub options: Option<Document>,
    }

    async fn get_db_collection(db_name: &str, coll_name: &str) -> (Database, Collection<Document>) {
        let client = mdb::get_client().await;
        let db = client.database(db_name);
        let collection = db.collection::<Document>(coll_name);

        (db, collection)
    }

    fn get_body_from_struct(structure: impl Serialize) -> Body {
        let structure_json = structure
            .serialize(bson::Serializer::new())
            .unwrap()
            .into_canonical_extjson();

        Body::from(serde_json::to_vec(&structure_json).unwrap())
    }

    async fn get_doc_from_body(body: Body) -> Document {
        let body_bytes = to_bytes(body, usize::MAX).await.unwrap();
        let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
        let body_bson: Bson = body_json.try_into().unwrap();

        body_bson.as_document().unwrap().clone()
    }

    fn get_request(uri: &str, body: Body) -> Request<Body> {
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/ejson")
            .body(body)
            .unwrap()
    }

    #[tokio::test]
    async fn hello_world() {
        let db_name = format!("test-{}", ObjectId::new().to_string());
        let coll_name = String::from("users");
        let (db, collection) = get_db_collection(&db_name, &coll_name).await;

        let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };

        collection.delete_many(doc! {}).await.unwrap();
        collection.insert_one(&user).await.unwrap();

        let body = get_body_from_struct(FindOneBody {
            db: db_name,
            collection: coll_name,
            filter: doc! {"name": "john"},
            options: None,
        });

        let request = get_request("/findOne", body);
        let app_router = app::build().await;
        let response = app_router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let doc = get_doc_from_body(body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get_str("name"), user.get_str("name"));

        db.drop().await.unwrap();
    }
}
