use r2d2::PooledConnection;
use redis::Client;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgConnectOptions;
use sqlx::{PgPool, Pool, Postgres};
use tokio::io::AsyncReadExt;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    server: Option<ConfigServer>,
    database: Option<ConfigDatabase>,
    redis: Option<ConfigDatabase>,
    coc_api: Option<ConfigApi>,
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
            .expect("找不到config.yaml");
        let mut yaml_str = String::new();
        yaml_file
            .read_to_string(&mut yaml_str)
            .await
            .expect("read str error");
        serde_yml::from_str::<Config>(yaml_str.as_str()).expect("config error")
    }

    pub fn get_server(&self) -> ConfigServer {
        self.server.clone().unwrap_or_default()
    }

    pub fn get_database(&self) -> ConfigDatabase {
        self.database.clone().unwrap_or_default()
    }

    pub fn get_redis(&self) -> ConfigDatabase {
        self.redis.clone().unwrap_or_default()
    }

    pub fn get_api(&self) -> ConfigApi {
        self.coc_api.clone().unwrap_or_default()
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

    pub fn _get_url(&self) -> String {
        self.url
            .as_ref()
            .unwrap_or(&"0.0.0.0:50000".to_string())
            .clone()
    }

    pub fn get_path(&self) -> String {
        self.path.as_ref().unwrap_or(&"0.0.0.0".to_string()).clone()
    }
}

impl ConfigDatabase {
    pub async fn get(self) -> Pool<Postgres> {
        let option = PgConnectOptions::new()
            .host(&self.host.unwrap_or_default())
            .port(self.port.unwrap_or_default())
            .database(&self.name.unwrap_or_default())
            .username(&self.username.unwrap_or_default())
            .password(&self.password.unwrap_or_default());
        PgPool::connect_with(option).await.expect("connect error")
    }

    ///
    /// Redis
    /// Need redis connect url
    pub fn redis(self) -> PooledConnection<Client> {
        let client =
            Client::open(self.url.unwrap_or("redis://127.0.0.1/".parse().unwrap())).unwrap();
        let pool = r2d2::Pool::builder().build(client).unwrap();
        pool.get().unwrap()
    }
}
