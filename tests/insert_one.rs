mod helpers;

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use mongodb::{
        bson::{doc, oid::ObjectId, Document},
        error::WriteFailure,
    };
    use serde::{Deserialize, Serialize};

    use crate::helpers::{get_db_and_collection, get_struct_from_doc, one_shot};

    #[derive(Debug, Serialize, Deserialize)]
    struct InsertOneBody {
        pub db: String,
        pub collection: String,
        pub document: Document,
        pub options: Option<Document>,
    }

    #[tokio::test]
    async fn insert_one() {
        let (db, collection) = get_db_and_collection().await;
        let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };

        let body = InsertOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            document: user.clone(),
            options: None,
        };

        let (parts, doc) = one_shot("/insertOne", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get("inserted_id"), user.get("_id"));

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn insert_one_with_options() {
        let (db, collection) = get_db_and_collection().await;
        let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };

        let body = InsertOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            document: user.clone(),
            options: Some(doc! {
                "bypassDocumentValidation": true,
                "comment": "My insert one operation",
                "writeConcern": doc! {
                    "j": false,
                    "w": "majority",
                    "wtimeout": 200,
                }
            }),
        };

        let (parts, doc) = one_shot("/insertOne", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get("inserted_id"), user.get("_id"));

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn insert_one_error() {
        let (db, collection) = get_db_and_collection().await;
        let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };

        collection.insert_one(&user).await.unwrap();

        let body = InsertOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            document: user.clone(),
            options: None,
        };

        let (parts, doc) = one_shot("/insertOne", body).await;

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);

        let write_failure: WriteFailure = get_struct_from_doc::<WriteFailure>(doc);

        if let WriteFailure::WriteError(error) = write_failure {
            assert!(error.message.contains("E11000 duplicate key error"));
        } else {
            panic!("wrong")
        }

        db.drop().await.unwrap();
    }
}
