use crate::api::clan::ClanIconUrls;
use crate::util::Config;
use axum::http::header::AUTHORIZATION;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use void_log::{log_info, log_warn};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct War {
    pub reason: Option<String>,
    pub message: Option<String>,
    pub detail: Option<Value>,
    pub r#type: Option<String>,
    state: Option<String>,
    team_size: Option<i64>,
    attacks_per_member: Option<i64>,
    battle_modifier: Option<String>,
    preparation_start_time: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    pub clan: Option<WarClan>,
    pub opponent: Option<WarClan>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct WarClan {
    pub tag: Option<String>,
    pub name: Option<String>,
    badge_urls: Option<ClanIconUrls>,
    clan_level: Option<i64>,
    attacks: Option<i64>,
    stars: Option<i64>,
    destruction_percentage: Option<f64>,
    members: Option<Vec<WarClanMember>>,
    exp_earned: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct WarClanMember {
    tag: Option<String>,
    name: Option<String>,
    #[serde(rename = "townhallLevel")]
    town_hall_level: Option<i64>,
    map_position: Option<i64>,
    opponent_attacks: Option<i64>,
}

impl War {
    pub async fn get(tag: &str) -> Self {
        let coc_api = Config::get().await.get_api();
        let token = format!("Bearer {}", coc_api.token.unwrap_or_default());
        let tag = tag.replace("#", "").to_uppercase();
        log_info!("查询标签 #{}", &tag);
        let url = format!("https://api.clashofclans.com/v1/clans/%23{tag}/currentwar");
        log_info!("API {}", &url);
        let response = Client::new()
            .get(url)
            .header(AUTHORIZATION, token)
            .send()
            .await;
        match response {
            Ok(re) => {
                re.json::<Self>().await.unwrap_or_default()
            }
            Err(e) => {
                log_warn!("War {e}");
                Default::default()
            }
        }
    }
}

#[tokio::test]
async fn test_get_war() {
    War::get("#2G2GJRQQJ").await;
}

#[tokio::test]
async fn test_get_war_clan() {
    let coc_api = Config::get().await.get_api();
    let token = format!("Bearer {}", coc_api.token.unwrap_or_default());
    let tag = "#2LUUU8QP8";
    log_info!("查询标签 #{}", &tag);
    let url = format!("https://api.clashofclans.com/v1/clans/%23{tag}/currentwar");
    log_info!("API {}", &url);
    let response = Client::new()
        .get(url)
        .header(AUTHORIZATION, token)
        .send()
        .await;
    let a = response.unwrap().text().await.unwrap();
    log_info!("{a}")
}
