use axum::{routing::post, Router};

use crate::{crud, mdb};

pub async fn build() -> Router {
    let client = mdb::get_client().await;

    Router::new()
        .route("/find", post(crud::find::handler))
        .route("/findOne", post(crud::find_one::handler))
        .route("/insertMany", post(crud::insert_many::handler))
        .route("/insertOne", post(crud::insert_one::handler))
        .with_state(client)
}
