use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, Error, FromRow, Pool, Postgres};
use uuid::Uuid;
use crate::system::Group;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Role {
    id: Option<Uuid>,
    name: String,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    update_time: DateTime<Utc>,
    status: Option<i16>,
    code: Option<String>,
}

impl Group {
    pub async fn group_roles(&self, pool: &Pool<Postgres>) -> Result<Vec<Role>, Error> {
        query_as("select * from public.role r, public.group_role rg where r.id = ug.role_id and ug.group_id = $1")
            .bind(&self.get_id()).fetch_all(pool).await
    }
}