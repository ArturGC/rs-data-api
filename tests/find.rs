mod helpers;

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use mongodb::{
        bson::{doc, oid::ObjectId, Document},
        error::CommandError,
    };
    use serde::{Deserialize, Serialize};

    use crate::helpers::{
        get_db_and_collection, get_struct_from_doc, one_shot_array, one_shot_document,
    };

    #[derive(Serialize, Deserialize)]
    struct FindBody {
        pub db: String,
        pub collection: String,
        pub filter: Document,
        pub options: Option<Document>,
    }

    #[tokio::test]
    async fn find() {
        let (db, collection) = get_db_and_collection().await;
        let user_0 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let user_1 = doc! { "_id": ObjectId::new(), "name": "jim", "age": 30 };

        collection.insert_many([&user_0, &user_1]).await.unwrap();

        let body = FindBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {"age": 30},
            options: None,
        };

        let (parts, doc) = one_shot_array("/find", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.get(0).unwrap().as_document().unwrap(), &user_0);
        assert_eq!(doc.get(1).unwrap().as_document().unwrap(), &user_1);

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn find_with_options() {
        let (db, collection) = get_db_and_collection().await;
        let user_0 = doc! { "_id": ObjectId::new(), "name": "john", "age": 30 };
        let user_1 = doc! { "_id": ObjectId::new(), "name": "jim", "age": 31 };
        let user_2 = doc! { "_id": ObjectId::new(), "name": "jones", "age": 32 };

        collection
            .insert_many([&user_0, &user_1, &user_2])
            .await
            .unwrap();

        let body = FindBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {},
            options: Some(doc! {
                "projection": doc! {"_id": 0, "name": 1},
                "sort": doc! {"age": 1},
                "skip": 1,
                "limit": 1,
            }),
        };

        let (parts, doc) = one_shot_array("/find", body).await;

        assert_eq!(parts.status, StatusCode::ACCEPTED);
        assert_eq!(doc.len(), 1);
        assert_eq!(
            doc.get(0).unwrap().as_document().unwrap(),
            &doc! {"name": "jim"}
        );

        db.drop().await.unwrap();
    }

    #[tokio::test]
    async fn find_error() {
        let (db, collection) = get_db_and_collection().await;

        let body = FindBody {
            db: db.name().into(),
            collection: collection.name().into(),
            filter: doc! {"age": doc!{"$in": {}}},
            options: None,
        };

        let (parts, doc) = one_shot_document("/find", body).await;

        assert_eq!(parts.status, StatusCode::BAD_REQUEST);

        let error = get_struct_from_doc::<CommandError>(doc);

        assert!(error.message.contains("$in needs an array"));

        db.drop().await.unwrap();
    }
}
