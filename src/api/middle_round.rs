use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Pool, Postgres};
use void_log::log_info;
use crate::orange::Round;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
pub struct MiddleRoundApi {
    current_round: String,
    current_round_sync_time: String,
    future_round: String,
    future_round_sync_time: String,
}

impl MiddleRoundApi {
    pub async fn get() -> Self {
        let url = Url::parse("http://www.cocbzlm.com/api/gettime_global.php").unwrap();
        log_info!("{}", &url);

        let response = Client::new()
            .post(url).header("x-api-key", "AFDFSDSaawdfFFeeeAAFDAGHHJNH996!!")
            .send().await;
        response.expect("response error").json().await.expect("failed to parse API response")
    }

    pub async fn _get_text() -> Value {
        let url = Url::parse("http://www.cocbzlm.com/api/gettime_global.php").unwrap();
        log_info!("{}", &url);

        let response = Client::new()
            .post(url).header("x-api-key", "AFDFSDSaawdfFFeeeAAFDAGHHJNH996!!")
            .send().await;
        response.expect("response error").json().await.expect("failed to parse API response")
    }
    
    pub async fn new_round(&self, pool: &Pool<Postgres>) -> u64 {
        if self.current_round.eq(&self.future_round) { 
            return 0;
        }
        Round::insert(&self.future_round, pool).await.unwrap_or_default().rows_affected()
    }
}

#[tokio::test]
async fn test() {
    let a = MiddleRoundApi::_get_text().await;
    println!("{a:?}")
}
