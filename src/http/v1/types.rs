use crate::database::{
    self,
    models::{user::UserId, user_session::PIDUserSessionId},
};

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct User {
    pub id: UserId,
    pub login: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::models::user::User> for User {
    fn from(value: database::models::user::User) -> Self {
        Self {
            id: value.id,
            login: value.login,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]

pub struct Session {
    pub id: PIDUserSessionId,
    pub active_expires_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<database::models::user_session::UserSession> for Session {
    fn from(value: database::models::user_session::UserSession) -> Self {
        Self {
            id: value.pid,
            active_expires_at: value.active_expires_at,
            expires_at: value.expires_at,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
