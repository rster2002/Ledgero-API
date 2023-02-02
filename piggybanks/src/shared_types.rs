use crate::models::service::jwt_service::JwtService;
use rocket::State;
use sqlx::{Pool, Postgres};

pub type DbPool = Pool<Postgres>;

pub type SharedPool = State<DbPool>;
pub type SharedJwtService = State<JwtService>;
