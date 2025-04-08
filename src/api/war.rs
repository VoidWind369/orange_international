use crate::api::clan::ClanIconUrls;
use crate::util::Config;
use axum::http::header::AUTHORIZATION;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use void_log::{log_info, log_warn};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct War {
    state: Option<String>,
    team_size: Option<i64>,
    attacks_per_member: Option<i64>,
    battle_modifier: Option<String>,
    preparation_start_time: Option<String>,
    start_time: Option<String>,
    end_time: Option<String>,
    clan: Option<WarClan>,
    opponent: Option<WarClan>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct WarClan {
    tag: Option<String>,
    name: Option<String>,
    badge_urls: Vec<ClanIconUrls>,
    clan_level: Option<i64>,
    attacks: Option<i64>,
    stars: Option<i64>,
    destruction_percentage: Option<i64>,
    members:Vec<WarClanMember>
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
struct WarClanMember {
    tag: Option<String>,
    name: Option<String>,
    #[serde(rename(deserialize = "townhallLevel"))]
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
        let response = Client::new().get(url)
            .header(AUTHORIZATION, token).send().await;
        match response {
            Ok(re) => {
                let data = re.json().await.unwrap_or_default();
                log_info!("{:?}", data);
                data
            }
            Err(e) => {
                log_warn!("War {e}");
                Default::default()
            }
        }
    }
}