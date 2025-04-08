use crate::orange::Clan;
use crate::system::{Group, User};
use crate::util::Config;
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;
use void_log::log_info;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserInfo {
    id: Uuid,
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

    pub async fn set_user(&self, ex_time: u64) {
        let json_str = serde_json::to_string(&self).unwrap();
        let config = Config::get().await.get_redis();
        let mut conn = config.redis();
        let _: () = conn.set_ex(&self.id, json_str, ex_time).unwrap();
    }

    pub async fn get_user(key: &str) -> serde_json::Result<Self> {
        let config = Config::get().await.get_redis();
        let mut conn = config.redis();
        let json_str = conn.get::<_, String>(key).unwrap();
        serde_json::from_str(&json_str)
    }
}

#[tokio::test]
async fn test_set_user() {
    let u = UserInfo {
        id: Uuid::from_str("a036b14c-9f83-4369-9086-3a82c0c8f05e").unwrap(),
        code: "".to_string(),
        email: "".to_string(),
        name: "".to_string(),
        token: "qwertyuiol;lkjhgfd".to_string(),
        group: vec![],
        clans: vec![],
    };
    u.set_user(3600).await;
}

#[tokio::test]
async fn test_get_user() {
    let a = UserInfo::get_user("test")
        .await
        .expect("TODO: panic message");
    log_info!("{a:?}")
}
