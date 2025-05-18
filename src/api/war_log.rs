use axum::http::header::AUTHORIZATION;
use chrono::{DateTime, NaiveDateTime, Utc};
use reqwest::Client;
use crate::api::war::WarClan;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use void_log::{log_info, log_link, log_warn};
use crate::util::Config;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WarLog {
    reason: Option<String>,
    message: Option<String>,
    r#type: Option<String>,
    detail: Option<Value>,
    items: Option<WarLogItems>,
    paging: Option<WarLogPaging>,
}

type WarLogItems = Vec<WarLogItem>;
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WarLogItem {
    result: String,
    end_time: String,
    team_size: u8,
    attacks_per_member: u64,
    battle_modifier: String,
    clan: WarClan,
    opponent: WarClan,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WarLogPaging {
    cursors: WarLogPagingCursors,
    
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WarLogPagingCursors {
    after: String,

}

impl WarLog {
    pub async fn get(tag: &str, limit: u64) -> Self {
        let coc_api = Config::get().await.get_api();
        let token = format!("Bearer {}", coc_api.token.unwrap_or_default());
        let tag = tag.replace("#", "").to_uppercase();
        log_info!("查询标签 #{}", &tag);
        let url = format!("https://api.clashofclans.com/v1/clans/%23{tag}/warlog?limit={limit}");
        log_info!("API {}", &url);
        let response = Client::new()
            .get(url)
            .header(AUTHORIZATION, token)
            .send()
            .await;
        match response {
            Ok(re) => {
                re.json::<Self>().await.unwrap()
            }
            Err(e) => {
                log_warn!("WarLog {e}");
                Default::default()
            }
        }
    }
}

impl WarLogItem {
    pub fn end_time_utc(&self) -> DateTime<Utc> {
        let dt = NaiveDateTime::parse_from_str(&self.end_time, "%Y%m%dT%H%M%S%.3fZ").unwrap();
        dt.and_utc()
    }
}

#[tokio::test]
async fn test() {
    let w = WarLog::get("#q82u2qr9",15).await;
    log_link!("{:?}", w.clone());
    w.items.unwrap().iter().for_each(|i| {
        log_link!("{}", &i.end_time);
        let tz = i.end_time_utc();
        log_link!("{tz}")
    });
}
