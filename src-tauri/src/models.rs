use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: Uuid,
    pub text: String,
    pub completed: bool,
    pub deadline: Option<DateTime>,
    pub repeat: Option<DateTime>,
    pub auto_expire: bool,
    pub info: Option<String>,
    #[serde(default)]
    pub updated_at: u64,
}
