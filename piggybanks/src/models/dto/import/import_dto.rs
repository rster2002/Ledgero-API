use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDto {
    pub id: String,
    pub imported_at: String,
    pub filename: String,
}
