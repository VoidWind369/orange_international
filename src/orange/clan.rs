use crate::api;
use crate::orange::clan_point::ClanPoint;
use crate::system::User;
use crate::util::Config;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{Error, FromRow, Pool, Postgres, query, query_as};
use uuid::Uuid;
use void_log::log_info;

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct Clan {
    pub id: Option<Uuid>,
    pub tag: Option<String>,
    pub name: Option<String>,
    #[serde(skip_deserializing)]
    create_time: DateTime<Utc>,
    #[serde(skip_deserializing)]
    update_time: DateTime<Utc>,
    pub status: Option<i16>,
    pub series_id: Option<Uuid>,
    is_global: Option<bool>,
}

impl Clan {
    pub async fn select_all(pool: &Pool<Postgres>) -> Result<Vec<Self>, Error> {
        query_as("select * from orange.clan").fetch_all(pool).await
    }

    pub async fn select(pool: &Pool<Postgres>, id: Uuid) -> Result<Self, Error> {
        query_as("select * from orange.clan where id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
    }

    pub async fn select_tag(
        pool: &Pool<Postgres>,
        tag: &str,
        is_intel: bool,
    ) -> Result<Self, Error> {
        query_as("select * from orange.clan where tag = $1 and is_global = $2 and status = 1")
            .bind(tag)
            .bind(is_intel)
            .fetch_one(pool)
            .await
    }

    pub async fn clan_users(&self, pool: &Pool<Postgres>) -> Result<Vec<User>, Error> {
        query_as("select * from public.user u, orange.clan_user cu where u.id = cu.user_id  and cu.clan_id = $1")
            .bind(&self.id).fetch_all(pool).await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("insert into orange.clan values(DEFAULT, $1, $2, $3, $3, $4, $5)")
            .bind(&self.tag)
            .bind(&self.name)
            .bind(now)
            .bind(&self.status)
            .bind(&self.series_id)
            .execute(pool)
            .await
    }

    pub async fn update(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update orange.clan set tag = $1, name = $2, update_time = $3, series_id = $4 where id = $5")
            .bind(&self.tag)
            .bind(&self.name)
            .bind(now)
            .bind(&self.series_id)
            .bind(&self.id)
            .execute(pool)
            .await
    }
    
    pub async fn update_status(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let now = Utc::now();
        query("update orange.clan set update_time = $1, status = $2 where id = $3")
            .bind(now)
            .bind(&self.status)
            .bind(&self.id)
            .execute(pool)
            .await
    }

    pub async fn delete(pool: &Pool<Postgres>, id: Uuid) -> Result<PgQueryResult, Error> {
        ClanUser::delete_clan(id, pool).await?;
        ClanPoint::delete(pool, id).await?;
        query("delete from orange.clan where id = $1")
            .bind(id)
            .execute(pool)
            .await
    }

    /// # 接口自动更新
    pub async fn api_insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let tag = &self.tag.clone().unwrap_or_default();
        let clan = api::Clan::get(tag).await.api_to_orange();
        clan.insert(pool).await
    }
    
    pub async fn api_update(&mut self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        let tag = &self.tag.clone().unwrap_or_default();
        let clan = api::Clan::get(tag).await.api_to_orange();
        self.tag = clan.tag;
        self.name = clan.name;
        self.update(pool).await
    }
}

// 关联User
impl User {
    ///
    ///
    /// # Clan Users
    ///
    /// * `pool`:
    ///
    /// returns: Result<Vec<Clan, Global>, Error>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub async fn user_clans(&self, pool: &Pool<Postgres>) -> Result<Vec<Clan>, Error> {
        log_info!("{:?}", &self.id);
        query_as("select c.* from orange.clan c, orange.clan_user cu where c.id = cu.clan_id and cu.user_id = $1")
            .bind(&self.id).fetch_all(pool).await
    }
}

#[derive(Debug, Clone, PartialEq, Default, FromRow, Serialize, Deserialize)]
pub struct ClanUser {
    pub clan_id: Uuid,
    pub user_id: Uuid,
}

impl ClanUser {
    pub async fn select(&self, pool: &Pool<Postgres>) -> Result<Self, Error> {
        query_as("select * from orange.clan_user where clan_id = $1 and user_id = $2")
            .fetch_one(pool)
            .await
    }

    pub async fn insert(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("insert into orange.clan_user values ($1, $2)")
            .bind(self.clan_id)
            .bind(self.user_id)
            .execute(pool)
            .await
    }

    pub async fn delete(&self, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from orange.clan_user where clan_id = $1 and user_id = $2")
            .bind(self.clan_id)
            .bind(self.user_id)
            .execute(pool)
            .await
    }

    pub async fn delete_user(user_id: Uuid, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from orange.clan_user where user_id = $1")
            .bind(user_id)
            .execute(pool)
            .await
    }

    pub async fn delete_clan(clan_id: Uuid, pool: &Pool<Postgres>) -> Result<PgQueryResult, Error> {
        query("delete from orange.clan_user where clan_id = $1")
            .bind(clan_id)
            .execute(pool)
            .await
    }
}

impl api::Clan {
    pub fn api_to_orange(&self) -> Clan {
        Clan {
            tag: (&self).tag.clone(),
            name: (&self).name.clone(),
            status: Some(1),
            ..Default::default()
        }
    }
}

#[tokio::test]
async fn test() {
    let pool = Config::get().await.get_database().get().await;
    let a = Clan::select_all(&pool).await.unwrap();
    log_info!("{:?}", a)
}
