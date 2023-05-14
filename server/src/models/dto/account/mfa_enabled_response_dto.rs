use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaEnabledResponseDto {
    pub backup_codes: [String; 8],
}
