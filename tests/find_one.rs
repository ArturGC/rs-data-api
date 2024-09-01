mod helpers;

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use mongodb::{
        bson::{doc, oid::ObjectId, Document},
        error::CommandError,
    };
    use serde::{Deserialize, Serialize};

    use crate::helpers::{get_db_and_collection, get_struct_from_doc, one_shot_document};

    #[derive(Serialize, Deserialize)]
    struct FindOneBody {
        pub db: String,
        pub collection: String,
        pub filter: Document,
        pub options: Option<Document>,
    }

    #[tokio::test]
    async fn find_one() {
        let (db, collection) = get_db_and_collection().await;

        let user = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };

        collection.insert_one(&user).await.unwrap();

        let body = FindOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {"name": "john"},
            options: None,
        };

        let (parts, doc) = one_shot_document("/findOne", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get_str("name"), user.get_str("name"));

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn find_one_with_options() {
        let (db, collection) = get_db_and_collection().await;

        let user_1 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let user_2 = doc! { "_id": ObjectId::new(), "name": "ane", "age": 30 };

        collection.insert_many([&user_1, &user_2]).await.unwrap();

        let body = FindOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {"age": 30},
            options: Some(doc! {
                "projection": doc! {"_id": 0, "name": 1},
                "sort": doc! {"name": 1}
            }),
        };

        let (parts, doc) = one_shot_document("/findOne", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc, doc! {"name": "ane"});

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn find_one_error() {
        let (db, collection) = get_db_and_collection().await;

        let body = FindOneBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {"age": doc!{"$in": {}}},
            options: None,
        };

        let (parts, doc) = one_shot_document("/findOne", body).await;

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);

        let error = get_struct_from_doc::<CommandError>(doc);

        assert!(error.message.contains("$in needs an array"));

        db.drop().await.unwrap();
    }
}
