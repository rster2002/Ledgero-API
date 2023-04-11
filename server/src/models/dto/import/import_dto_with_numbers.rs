use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDtoWithNumbers {
    pub id: String,
    pub imported_at: String,
    pub filename: String,
    pub imported: i32,
    pub skipped: i32,
}
