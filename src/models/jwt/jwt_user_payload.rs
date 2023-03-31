use rocket::http::Status;
use rocket::outcome::Outcome::{Failure, Success};
use rocket::request::{FromRequest, Outcome};
use rocket::Request;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use crate::error::jwt_error::JwtError;
use crate::models::entities::user::user_role::UserRole;
use crate::prelude::*;
use crate::services::jwt_service::JwtService;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JwtUserPayload {
    pub uuid: String,
    pub username: String,
    pub role: UserRole,
}

impl Display for JwtUserPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({}, {})", self.username, self.role, self.uuid)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for JwtUserPayload {
    type Error = Error;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jwt_service = request
            .rocket()
            .state::<JwtService>()
            .expect("Failed to get jwt service");

        let authorization_value = request.headers().get("Authorization").next();

        let Some(bearer_value) = authorization_value else {
            return Failure((Status::Unauthorized, JwtError::MissingToken.into()));
        };

        if !bearer_value.starts_with("Bearer ") {
            return Failure((Status::Unauthorized, JwtError::MissingToken.into()));
        }

        let mut bearer_value = bearer_value.to_string();
        bearer_value = bearer_value.replace("Bearer ", "");

        let result = jwt_service.decode_access_token(bearer_value);

        let Ok(payload) = result else {
            return Failure((Status::Unauthorized, result.expect_err("Was not Ok, bot also not an error?")));
        };

        Success(payload)
    }
}
