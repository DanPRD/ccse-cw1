pub mod signin {

    use askama::Template;
    use axum::{http::StatusCode, response::{Html, IntoResponse}};

    #[derive(Template)]
    #[template(path="signin.html")]
    struct SignInPage {

    }

    pub async fn sign_in() -> impl IntoResponse {
        let template = SignInPage {};
        let html = template.render().unwrap();
        return (StatusCode::OK, Html(html))
    }

}

pub mod signup {

    use askama::Template;
    use axum::{http::{HeaderMap, StatusCode}, response::{AppendHeaders, Html, IntoResponse, Redirect}, Form};
    use serde::Deserialize;

    #[derive(Template)]
    #[template(path="signup.html")]
    struct SignUpPage {

    }

    #[derive(Debug, Deserialize)]
    pub struct SignUpData {
        username: String,
        email: String,
        password: String,
        password2: String
    }

    pub async fn sign_up() -> impl IntoResponse {
        let template = SignUpPage {};
        let html = template.render().unwrap();
        return (StatusCode::OK, Html(html))
    }

    pub async fn process_sign_up(Form(sign_up_form): Form<SignUpData>) -> impl IntoResponse {
        println!("Got Data: {:?}", sign_up_form);
        if true {
            return (StatusCode::BAD_REQUEST, "This is an Error Message").into_response();
        } else {
            return AppendHeaders([("HX-Redirect", "/")]).into_response();

        }
        
    }
}