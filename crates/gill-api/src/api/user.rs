use crate::error::AppError;
use activitypub_federation::core::signatures::generate_actor_keypair;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use gill_db::user::{CreateSSHKey, CreateUser, User};
use gill_settings::SETTINGS;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
}

pub async fn create(
    pool: Extension<PgPool>,
    Json(user): Json<CreateUserCommand>,
) -> Result<Response, AppError> {
    let keys = generate_actor_keypair()?;
    let scheme = if gill_settings::debug_mod() {
        "http://"
    } else {
        "https://"
    };
    let domain = &SETTINGS.domain;
    let username = user.username;

    let user = CreateUser {
        username: username.clone(),
        email: Some(user.email),
        private_key: Some(keys.private_key),
        public_key: keys.public_key,
        followers_url: format!("{scheme}{domain}/apub/{username}/followers"),
        outbox_url: format!("{scheme}{domain}/apub/{username}/outbox"),
        inbox_url: format!("{scheme}{domain}/apub/{username}/inbox"),
        activity_pub_id: format!("{scheme}{domain}/apub/users/{username}"),
        domain: SETTINGS.domain.clone(),
        is_local: true,
    };

    User::create(user, &pool.0).await?;
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

pub async fn register_ssh_key(
    Extension(user): Extension<User>,
    Extension(pool): Extension<PgPool>,
    Json(ssh_key): Json<CreateSSHKey>,
) -> Result<Response, AppError> {
    User::add_ssh_key(user.id, &ssh_key.key, &pool).await?;
    #[cfg(not(feature = "integration"))]
    gill_git::append_ssh_key(&ssh_key.key).expect("Failed to append ssh key");
    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
