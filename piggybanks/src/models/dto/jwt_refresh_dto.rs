use rocket::serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct JwtRefreshDto<'a> {
    pub access_token: &'a str,
    pub refresh_token: &'a str,
}
