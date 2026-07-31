use crate::{api::clan::ClanIconUrls, util::Config};
use axum::http::header::AUTHORIZATION;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
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
    pub fn get_opponent(&self) -> WarClan {
        self.opponent.clone().unwrap_or_default()
    }

    pub fn get_preparation_start_time(&self) -> NaiveDateTime {
        let time = self.preparation_start_time.clone().unwrap_or_default();
        NaiveDateTime::parse_from_str(&time, "%Y%m%dT%H%M%S%.3fZ").unwrap()
    }

    pub fn check_preparation_start_time(&self, round_time: DateTime<Utc>) -> bool {
        let pst = self.get_preparation_start_time().and_utc();
        let add_rt = round_time.checked_add_signed(TimeDelta::hours(1)).unwrap();
        let sub_rt = round_time
            .checked_sub_signed(TimeDelta::minutes(5))
            .unwrap();
        pst < add_rt || pst > sub_rt
    }

    pub fn check_preparation_start_time_error(&self, round_time: DateTime<Utc>) -> bool {
        let pst = self.get_preparation_start_time();
        let add_pst = pst
            .checked_add_signed(TimeDelta::hours(20))
            .unwrap()
            .and_utc();
        round_time > add_pst
    }

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
            Ok(re) => re.json::<Self>().await.unwrap_or_default(),
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

#[test]
fn test_time() {
    let dt = NaiveDateTime::parse_from_str("20260729T081205.000Z", "%Y%m%dT%H%M%S%.3fZ").unwrap();
    let utc_dt = DateTime::<Utc>::from_utc(dt, Utc);
    log_info!("{}", utc_dt);
    log_info!("{}", dt.and_utc())
}
