use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum JwtTokenType {
    Access,
    Refresh,
}
