use crate::middle;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use void_log::log_info;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MiddleTrackApi {
    pub server: String,
    #[serde(alias = "bzlm_total_score")]
    pub bz_total_score: i64,
    pub public_total_score: i64,
    pub details: Vec<MiddleTrackApiDetails>,
    pub summary: Vec<String>,
    #[serde(skip_deserializing)]
    pub tag: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
// #[serde(rename_all(deserialize = "camelCase"))]
pub struct MiddleTrackApiDetails {
    // #[serde(rename(deserialize = "bzlmRound"))]
    #[serde(alias = "bzlmRound")]
    pub bz_round: i64,
    // #[serde(rename = "round_point")]
    pub round_point: i64,
    #[serde(alias = "roundResult")]
    pub round_result: String,
    #[serde(alias = "clanTag")]
    pub clan_tag: String,
    #[serde(alias = "oppClanTag")]
    pub opp_clan_tag: String,
    pub explain: String,
}

impl MiddleTrackApi {
    pub async fn get(tag: &str) -> Self {
        let tag = format!("#{}", tag.replace("#", "").to_uppercase());
        let mut url = Url::parse("http://cocbzlm.com:8422/api/accinfo/scores").unwrap();
        url.query_pairs_mut().append_pair("clanTag", &tag);
        url.query_pairs_mut().append_pair("isGlobal", "true");
        log_info!("{}", &url);
        
        let response = reqwest::get(url).await;
        let mut api = response.expect("response error").json::<Self>().await.expect("failed to parse API response");
        api.tag = tag;
        api
    }
    
    pub fn self_to_database(self) -> middle::Track {
        middle::Track {
            server: self.server,
            bz_total_score: self.bz_total_score,
            public_total_score: self.public_total_score,
            details: Json(self.details),
            summary: self.summary,
            tag: self.tag,
            ..Default::default()
        }
    }
}

#[tokio::test]
async fn test() {
    
    let pool = crate::util::Config::get().await.get_database().get().await;
    let a = MiddleTrackApi::get("#2J9999990").await.self_to_database();
    let b = a.insert(&pool).await.unwrap();
    log_info!("{a:?} {}", b.rows_affected());
}
