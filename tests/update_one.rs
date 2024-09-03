mod helpers;

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use mongodb::{
        bson::{doc, oid::ObjectId, Document},
        error::WriteFailure,
    };
    use serde::{Deserialize, Serialize};

    use crate::helpers::{get_db_and_collection, get_struct_from_doc, one_shot_document};

    #[derive(Debug, Serialize, Deserialize)]
    struct UpdateOneBody {
        pub db: String,
        pub collection: String,
        pub query: Document,
        pub update: Document,
        pub options: Option<Document>,
    }

    #[tokio::test]
    async fn update_one() {
        let (db, collection) = get_db_and_collection().await;
        let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let query = doc! {"name": user.get_str("name").unwrap()};
        let update = doc! {"$set": doc! {"age": 35}};

        collection.insert_one(&user).await.unwrap();

        let body = UpdateOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            query,
            update,
            options: None,
        };

        let (parts, doc) = one_shot_document("/updateOne", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get_i64("matchedCount").unwrap(), 1);
        assert_eq!(doc.get_i64("modifiedCount").unwrap(), 1);

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn update_one_upsert() {
        let (db, collection) = get_db_and_collection().await;
        let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let query = doc! {"name": user.get_str("name").unwrap()};
        let update = doc! {"$setOnInsert": user.clone()};
        let options = Some(doc! {"upsert": true});

        let body = UpdateOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            query,
            update,
            options,
        };

        let (parts, doc) = one_shot_document("/updateOne", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get_i64("matchedCount").unwrap(), 0);
        assert_eq!(doc.get_i64("modifiedCount").unwrap(), 0);
        assert_eq!(doc.get("upsertedId"), user.get("_id"));

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn update_one_error() {
        let (db, collection) = get_db_and_collection().await;
        let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let query = doc! {"name": user.get_str("name").unwrap()};
        let update = doc! {"$setOrInsert": user.clone()};
        let options = Some(doc! {"upsert": true});

        let body = UpdateOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            query,
            update,
            options,
        };

        let (parts, doc) = one_shot_document("/updateOne", body).await;
        let error_message = match get_struct_from_doc::<WriteFailure>(doc) {
            WriteFailure::WriteConcernError(e) => e.message,
            WriteFailure::WriteError(e) => e.message,
            _ => "nothing".to_string(),
        };

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);
        assert!(error_message.contains("Unknown modifier"));

        db.drop().await.unwrap();
    }
}
