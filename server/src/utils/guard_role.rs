use jumpdrive_auth::errors::jwt_error::JwtError;
use crate::models::entities::user::user_role::UserRole;
use crate::models::jwt::jwt_user_payload::JwtUserPayload;
use crate::prelude::*;

pub fn guard_user_payload(user: &JwtUserPayload, required_role: UserRole) -> Result<()> {
    let result = guard_role(&user.role, &required_role);

    if result.is_err() {
        info!(
            "User {} failed to perform an action that required role '{}'",
            user, required_role
        );
    }

    result
}

pub fn guard_role(user_role: &UserRole, required_role: &UserRole) -> Result<()> {
    if user_role < required_role {
        return Err(JwtError::NotEnoughPermissions.into());
    }

    Ok(())
}
