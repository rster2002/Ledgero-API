use rocket::State;
use sqlx::{Pool, Postgres};
use crate::models::service::jwt_service::JwtService;

pub type SharedPool = State<Pool<Postgres>>;
pub type SharedJwtService = State<JwtService>;
