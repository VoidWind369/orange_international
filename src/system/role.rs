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
    status: Option<i16>,
    path: String,
    code: Option<String>,
}

impl Role {
    pub fn get_code(&self) -> String {
        self.code.clone().unwrap_or_default()
    }
}

impl Group {
    pub async fn group_roles(&self, pool: &Pool<Postgres>) -> Result<Vec<Role>, Error> {
        query_as("select * from public.role r, public.group_role rg where r.id = rg.role_id and rg.group_id = $1")
            .bind(&self.get_id()).fetch_all(pool).await
    }
}