use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Series {
    id: Uuid,
    name: Option<String>,
    #[serde(skip_deserializing)]
    create_time: Timestamp,
    #[serde(skip_deserializing)]
    update_time: Timestamp,
    status: Option<i16>,
    code: Uuid,
}
