use crate::error::jwt_error::JwtError;
use crate::models::entities::user::user_role::UserRole;
use crate::prelude::*;

pub fn guard_role(user_role: &UserRole, required_role: UserRole) -> Result<()> {
    if user_role < &required_role {
        return Err(JwtError::NotEnoughPermissions.into());
    }

    Ok(())
}
