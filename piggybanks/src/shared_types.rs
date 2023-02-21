use rocket::State;
use sqlx::{Pool, Postgres};
use crate::services::jwt_service::JwtService;

pub type DbPool = Pool<Postgres>;

pub type SharedPool = State<DbPool>;
pub type SharedJwtService = State<JwtService>;
