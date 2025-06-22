use crate::orange::Clan;
use crate::system::{Group, User};
use crate::util::Config;
use redis::{Commands, RedisResult, Value};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    code: String,
    email: String,
    name: String,
    token: String,
    group: Vec<Group>,
    clans: Vec<Clan>,
}

impl UserInfo {
    pub fn new(user: User, token: String, clans: Vec<Clan>, group: Vec<Group>) -> Self {
        Self {
            id: user.id.unwrap(),
            code: user.code.unwrap(),
            email: user.email.unwrap(),
            name: user.name.unwrap_or_default(),
            token,
            group,
            clans,
        }
    }

    pub fn get_token(&self) -> String {
        self.token.clone()
    }

    pub fn get_id(&self) -> Uuid {
        self.id.clone()
    }

    pub async fn set_user(&self, ex_time: u64) {
        let json_str = serde_json::to_string(&self).unwrap();
        let config = Config::get().await.get_redis();
        let mut conn = config.redis();
        let _: () = conn.set_ex(&self.token, json_str, ex_time).unwrap();
    }

    pub async fn get_user(key: &str) -> serde_json::Result<Self> {
        let config = Config::get().await.get_redis();
        let mut conn = config.redis();
        let json_str = conn.get::<_, String>(key).unwrap_or_default();
        serde_json::from_str(&json_str)
    }

    pub async fn del_user(key: &str) -> RedisResult<Value> {
        let config = Config::get().await.get_redis();
        let mut conn = config.redis();
        conn.del(key)
    }
}
