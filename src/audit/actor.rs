use std::fmt::Display;

use crate::{audit::SYSTEM_USER_ID, database::models::user::UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditActor {
    System,
    Admin(UserId),
    UserSelf(UserId),
}

impl AuditActor {
    pub fn id(self) -> UserId {
        match self {
            Self::Admin(v) | Self::UserSelf(v) => v,
            Self::System => SYSTEM_USER_ID,
        }
    }
}

impl Display for AuditActor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Admin(_) => write!(f, "admin"),
            Self::UserSelf(_) => write!(f, "user_self"),
            Self::System => write!(f, "system"),
        }
    }
}
