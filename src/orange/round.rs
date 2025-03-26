use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Round {
    id: Uuid,
    code: String,
    round_time: DateTime<Utc>,
    create_time: DateTime<Utc>,
}