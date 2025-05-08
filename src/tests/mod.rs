use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use axum_test::TestServer;
use tower::ServiceExt;

use crate::create_srv;
/*
Test User credentials exist in the database already, they should of been created on first start

*/


#[tokio::test]
async fn server_runs() {
    let srv = create_srv().await;
    let response = srv
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK)
}

#[tokio::test]
async fn login_wrong_username() {
    let srv = create_srv().await;
    let response = srv
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sign-in")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=randomemail&password=randompassword"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
}

#[tokio::test]
async fn login_wrong_password() {
    let srv = create_srv().await;
    let response = srv
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sign-in")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=testemail@securecart.com&password=mywrongpassword",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED)
}

#[tokio::test]
async fn login_correct() {
    let srv = create_srv().await;
    let response = srv
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/sign-in")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(
                    "email=testemail@securecart.com&password=mysecurepassword",
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK)
}

#[tokio::test]
async fn logout() {
    let creds = [
        ("email", "testemail@securecart.com"),
        ("password", "mysecurepassword"),
    ];
    let srv = TestServer::new(create_srv().await).unwrap();
    let response = srv
        .post("/sign-in")
        .form(&creds)
        .save_cookies()
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let logout_resp = srv.post("/sign-out").await;
    assert_eq!(logout_resp.status_code(), StatusCode::OK);
}

async fn login_success() {}
