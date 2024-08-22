use reqwest::{
    header::{self},
    StatusCode,
};

#[tokio::test]
async fn no_header_value() {
    let client = reqwest::Client::builder()
        .default_headers(header::HeaderMap::new())
        .build()
        .unwrap();

    let res = client
        .post("http://127.0.0.1:8080/findOne")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "Content Type not found");
}

#[tokio::test]
async fn not_str_header_value() {
    let client = reqwest::Client::builder()
        .default_headers(header::HeaderMap::new())
        .build()
        .unwrap();

    let res = client
        .post("http://127.0.0.1:8080/findOne")
        .header(header::CONTENT_TYPE, "🤷‍♂️")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        res.text().await.unwrap(),
        "failed to convert header to a str"
    );
}

#[tokio::test]
async fn wrong_header_value() {
    let client = reqwest::Client::builder()
        .default_headers(header::HeaderMap::new())
        .build()
        .unwrap();

    let res = client
        .post("http://127.0.0.1:8080/findOne")
        .header(header::CONTENT_TYPE, "application/wrong")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "Content Type not accepted");
}

#[tokio::test]
async fn not_ejson_body() {
    let client = reqwest::Client::builder()
        .default_headers(header::HeaderMap::new())
        .build()
        .unwrap();

    let res = client
        .post("http://127.0.0.1:8080/findOne")
        .header(header::CONTENT_TYPE, "application/ejson")
        .body("{ name: john }")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        res.text().await.unwrap(),
        "key must be a string at line 1 column 3"
    );
}

#[tokio::test]
async fn not_bson_body() {
    let client = reqwest::Client::builder()
        .default_headers(header::HeaderMap::new())
        .build()
        .unwrap();

    let res = client
        .post("http://127.0.0.1:8080/findOne")
        .header(header::CONTENT_TYPE, "application/ejson")
        .body("{ \"$numberLong\": 5 }")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert_eq!(res.text().await.unwrap(), "EJSON to BSON parse error");
}

#[tokio::test]
async fn wrong_struct_body() {
    let client = reqwest::Client::builder()
        .default_headers(header::HeaderMap::new())
        .build()
        .unwrap();

    let res = client
        .post("http://127.0.0.1:8080/findOne")
        .header(header::CONTENT_TYPE, "application/ejson")
        .body("{ \"name\": \"john\" }")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    assert!(res.text().await.unwrap().contains("missing field"));
}
