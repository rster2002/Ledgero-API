use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::cmp::Ordering;

/// The role of the user may allow for extra operation to be used throughout the application.
#[derive(Debug, Type, Serialize, Deserialize, PartialEq, Eq, Copy, Clone)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "camelCase")]
pub enum UserRole {
    /// This is the default role for a user. Allows them to use the application without any special
    /// permissions or perform special actions.
    User,

    /// Special user role that is allowed to do everything in the application, even special actions
    /// like creating or deleting users.
    System,
}

impl UserRole {
    pub fn get_score(&self) -> u8 {
        match self {
            UserRole::User => 1,
            UserRole::System => 2,
        }
    }
}

impl PartialOrd for UserRole {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_score = self.get_score();
        let other_score = other.get_score();

        Some(self_score.cmp(&other_score))
    }
}

impl From<String> for UserRole {
    fn from(value: String) -> Self {
        let str = &*value;

        if str == "system" {
            return UserRole::System;
        }

        UserRole::User
    }
}

impl Into<&str> for UserRole {
    fn into(self) -> &'static str {
        match self {
            UserRole::User => "user",
            UserRole::System => "system",
        }
    }
}
