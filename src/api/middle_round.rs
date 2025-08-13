use crate::orange::Round;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Pool, Postgres};
use void_log::log_info;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MiddleRoundApi {
    current_round: u64,
    current_sync_time: String,
    current_cfa_round: String,
    future_round: u64,
    future_sync_time: String,
    future_cfa_round: String,
}

impl MiddleRoundApi {
    pub async fn get() -> Self {
        let url =
            Url::parse("http://www.cocbzlm.com:8422/api/wardecider/syncTimeGlobal_alliance/cfa")
                .unwrap();
        log_info!("{}", &url);

        let response = Client::new()
            .get(url)
            .header("x-api-key", "AFDFSDSaawdfFFeeeAAFDAGHHJNH996!!")
            .send()
            .await;
        response
            .expect("response error")
            .json()
            .await
            .expect("failed to parse API response")
    }

    pub async fn _get_text() -> Value {
        let url = Url::parse("http://www.cocbzlm.com:8422/api/wardecider/syncTimeGlobal_alliance/cfa").unwrap();
        log_info!("{}", &url);

        let response = Client::new()
            .get(url)
            .header("x-api-key", "AFDFSDSaawdfFFeeeAAFDAGHHJNH996!!")
            .send()
            .await;
        response
            .expect("response error")
            .json()
            .await
            .expect("failed to parse API response")
    }

    pub async fn new_round(&self, pool: &Pool<Postgres>) -> Result<u64, String> {
        if self.current_round.eq(&self.future_round) {
            return Err("轮次未更新".to_string());
        }
        let res = Round::insert(&self.future_sync_time, pool)
            .await
            .unwrap_or_default()
            .rows_affected();
        Ok(res)
    }
}

#[tokio::test]
async fn test() {
    let a = MiddleRoundApi::get().await;
    println!("{a:?}")
}
