use crate::domain::user::User;
use crate::error::{AppError, AppResult};
use crate::get_connected_user_username;
use crate::oauth::Oauth2User;
use crate::view::HtmlTemplate;
use askama::Template;
use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Extension;
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct UserSettingsQuery {
    #[serde(default)]
    tab: Tab,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tab {
    SshKey,
    Profile,
}

impl Default for Tab {
    fn default() -> Self {
        Self::Profile
    }
}

#[derive(Template)]
#[template(path = "user/settings.html")]
pub struct UserSettingsTemplate {
    user: Option<String>,
    // TODO
    _tab: Tab,
}

pub async fn settings(
    connected_user: Option<Oauth2User>,
    Query(page): Query<UserSettingsQuery>,
    Extension(db): Extension<PgPool>,
) -> AppResult<impl IntoResponse> {
    let Some(user) = get_connected_user_username(&db, connected_user).await else {
        return Err(AppError::Unauthorized);
    };

    let user = User::by_name(&user, &db).await?;

    Ok(HtmlTemplate(UserSettingsTemplate {
        user: Some(user.username),
        _tab: page.tab,
    }))
}
