use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Series {
    id: Option<Uuid>,
    name: Option<String>,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    update_time: DateTime<Utc>,
    status: Option<i16>,
    code: Option<Uuid>,
}