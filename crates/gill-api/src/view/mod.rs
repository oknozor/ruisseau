use crate::oauth;
use crate::oauth::{AppState, Oauth2User};
use askama::Template;
use axum::http::StatusCode;
use axum::response::Response;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use gill_db::user::User;
use sqlx::PgPool;

pub mod index;
pub mod repositories;
pub mod repository;
pub mod user;

pub struct HtmlTemplate<T>(T);

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .merge(repository::routes())
        .merge(user::routes())
        .route("/", get(index::index))
        .route("/auth/gill/", get(oauth::openid_auth))
        .route("/auth/gill", get(oauth::openid_auth))
        .route("/auth/authorized/", get(oauth::login_authorized))
        .route("/auth/authorized", get(oauth::login_authorized))
        .route("/logout/", get(oauth::logout))
        .route("/repo/", get(repositories::list))
        .with_state(app_state)
}

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

async fn get_connected_user_username(db: &PgPool, user: Option<Oauth2User>) -> Option<String> {
    let email = user.map(|user| user.email);
    match email {
        Some(email) => User::by_email(&email, db).await.ok(),
        None => None,
    }
    .map(|user| user.username)
}
