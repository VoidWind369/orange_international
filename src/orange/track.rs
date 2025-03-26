use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct Track {
    id: Uuid,
    self_clan_id: Uuid,
    rival_clan_id: Uuid,
    self_history_point: i64,
    rival_history_point: i64,
    create_time: DateTime<Utc>,
    self_now_point: i64,
    rival_now_point: i64,
    round_id: Uuid,
}
