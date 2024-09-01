mod helpers;

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use mongodb::bson::{doc, oid::ObjectId, Document};
    use serde::{Deserialize, Serialize};

    use crate::helpers::{get_db_and_collection, one_shot_document};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct InsertManyBody {
        db: String,
        collection: String,
        documents: Vec<Document>,
        options: Option<Document>,
    }

    #[tokio::test]
    async fn insert_many() {
        let (db, collection) = get_db_and_collection().await;
        let user_0 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let user_1 = doc! { "_id": ObjectId::new(), "name": "jim", "age": 30 };

        let body = InsertManyBody {
            db: db.name().into(),
            collection: collection.name().into(),
            documents: vec![user_0.clone(), user_1.clone()],
            options: None,
        };

        let (parts, doc) = one_shot_document("/insertMany", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(
            doc.get_document("inserted_ids").unwrap().get("0"),
            user_0.get("_id")
        );

        assert_eq!(
            doc.get_document("inserted_ids").unwrap().get("1"),
            user_1.get("_id")
        );

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn insert_many_with_options() {
        let (db, collection) = get_db_and_collection().await;
        let user_0 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let user_1 = doc! { "_id": ObjectId::new(), "name": "jim", "age": 30 };

        let body = InsertManyBody {
            db: db.name().into(),
            collection: collection.name().into(),
            documents: vec![user_0.clone(), user_1.clone()],
            options: Some(doc! {
                "bypassDocumentValidation": true,
                "comment": "My insert one operation",
                "ordered": false,
                "writeConcern": doc! {
                    "j": false,
                    "w": "majority",
                    "wtimeout": 200,
                }
            }),
        };

        let (parts, doc) = one_shot_document("/insertMany", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(
            doc.get_document("inserted_ids").unwrap().get("0"),
            user_0.get("_id")
        );

        assert_eq!(
            doc.get_document("inserted_ids").unwrap().get("1"),
            user_1.get("_id")
        );

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn insert_many_error() {
        let (db, collection) = get_db_and_collection().await;
        let user_0 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let user_1 = doc! { "_id": ObjectId::new(), "name": "jim", "age": 30 };

        collection.insert_one(&user_1).await.unwrap();

        let body = InsertManyBody {
            db: db.name().into(),
            collection: collection.name().into(),
            documents: vec![user_0.clone(), user_1.clone()],
            options: Some(doc! {
                "ordered": false,
            }),
        };

        let (parts, doc) = one_shot_document("/insertMany", body).await;

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);

        let error_message = doc
            .get_array("writeErrors")
            .unwrap()
            .get(0)
            .unwrap()
            .as_document()
            .unwrap()
            .get_str("errmsg")
            .unwrap();

        assert!(error_message.contains("E11000 duplicate key error"));

        db.drop().await.unwrap();
    }
}
