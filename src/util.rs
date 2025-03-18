use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::{ConnectOptions, Error, PgPool, Pool, Postgres};
use tokio::io::AsyncReadExt;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    server: Option<ConfigServer>,
    database: Option<ConfigDatabase>,
    redis: Option<ConfigDatabase>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigServer {
    url: Option<String>,
    path: Option<String>,
    port: Option<u16>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ConfigDatabase {
    url: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    name: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ConfigApi {
    pub ws: Option<String>,
    pub url: Option<String>,
    pub id: Option<String>,
    pub account: Option<String>,
    pub token: Option<String>,
    pub secret: Option<String>,
}

impl Config {
    pub async fn get() -> Self {
        let mut yaml_file = tokio::fs::File::open("config.yaml")
            .await
            .expect("read config error");
        let mut yaml_str = String::new();
        yaml_file
            .read_to_string(&mut yaml_str)
            .await
            .expect("read str error");
        serde_yml::from_str::<Config>(yaml_str.as_str()).expect("config error")
    }

    pub fn get_server(&self) -> ConfigServer {
        self.server.as_ref().unwrap().clone()
    }

    pub fn get_database(&self) -> ConfigDatabase {
        self.database.as_ref().unwrap().clone()
    }

    pub fn get_redis(&self) -> ConfigDatabase {
        self.redis.as_ref().unwrap().clone()
    }
}

impl Default for ConfigServer {
    fn default() -> Self {
        Self {
            path: Some("0.0.0.0".to_string()),
            port: Some(50000),
            url: Some("0.0.0.0:50000".to_string()),
        }
    }
}

impl ConfigServer {
    pub fn get_port(&self) -> u16 {
        self.port.unwrap_or(50000)
    }

    pub fn get_url(&self) -> String {
        self.url.as_ref().unwrap_or("0.0.0.0:50000".as_ref()).clone()
    }

    pub fn get_path(&self) -> String {
        self.path.as_ref().unwrap_or("0.0.0.0".as_ref()).clone()
    }
}

impl ConfigDatabase {
    pub async fn get(self) -> Pool<Postgres> {
        let option = PgConnectOptions::new()
            .host(&self.host.unwrap_or_default())
            .port(self.port.unwrap_or_default())
            .username(&self.username.unwrap_or_default())
            .password(&self.password.unwrap_or_default())
            .ssl_mode(PgSslMode::Require);
        PgPool::connect_with(option).await.expect("connect error")
    }

    ///
    /// Redis
    /// Need redis connect url
    pub fn redis(self) {
        let con = redis::Client::open(&self.url);
    }
}
