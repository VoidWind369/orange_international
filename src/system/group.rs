use crate::system::User;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow, Pool, Postgres, query_as};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Group {
    id: Option<Uuid>,
    name: String,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    update_time: DateTime<Utc>,
    code: Option<String>,
}

impl Group {
    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from public.group").fetch_all(pool).await
    }

    pub async fn select(pool: &Pool<Postgres>, id: Uuid) -> Result<Self, Error> {
        query_as("select * from public.group where id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    pub async fn group_users(&self, pool: &Pool<Postgres>) -> Result<Vec<User>, Error> {
        query_as("select * from public.user u, public.user_group ug where u.id = ug.user_id  and ug.group_id = $1")
            .bind(&self.id).fetch_all(pool).await
    }
}

impl User {
    pub async fn user_groups(&self, pool: &Pool<Postgres>) -> Result<Vec<Group>, Error> {
        query_as("select * from public.group g, public.user_group ug where g.id = ug.group_id and ug.user_id = $1")
            .bind(&self.id).fetch_all(pool).await
    }
}
