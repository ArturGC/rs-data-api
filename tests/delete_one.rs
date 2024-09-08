mod helpers;

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use mongodb::{
        bson::{doc, oid::ObjectId, Document},
        error::WriteError,
    };
    use serde::{Deserialize, Serialize};

    use crate::helpers::{get_db_and_collection, get_struct_from_doc, one_shot_document};

    #[derive(Serialize, Deserialize)]
    struct DeleteManyBody {
        pub db: String,
        pub collection: String,
        pub filter: Document,
        pub options: Option<Document>,
    }

    #[tokio::test]
    async fn delete_one() {
        let (db, collection) = get_db_and_collection().await;

        let user_0 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let user_1 = doc! { "_id": ObjectId::new(), "name": "jim", "age": 30 };

        collection.insert_many([&user_0, &user_1]).await.unwrap();

        let body = DeleteManyBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {"age": 30},
            options: None,
        };

        let (parts, doc) = one_shot_document("/deleteOne", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get_i64("deletedCount"), Ok(1));

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn delete_one_with_options() {
        let (db, collection) = get_db_and_collection().await;

        let user_1 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let user_2 = doc! { "_id": ObjectId::new(), "name": "ane", "age": 30 };

        collection.insert_many([&user_1, &user_2]).await.unwrap();

        let body = DeleteManyBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {"age": 35},
            options: Some(doc! {
                "comment": String::from("Delete many operation test"),
            }),
        };

        let (parts, doc) = one_shot_document("/deleteOne", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get_i64("deletedCount"), Ok(0));

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn delete_one_error() {
        let (db, collection) = get_db_and_collection().await;

        let body = DeleteManyBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {"age": doc!{"$in": {}}},
            options: None,
        };

        let (parts, doc) = one_shot_document("/deleteOne", body).await;

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);

        let error =
            get_struct_from_doc::<WriteError>(doc.get_document("WriteError").unwrap().clone());

        assert!(error.message.contains("$in needs an array"));

        db.drop().await.unwrap();
    }
}
