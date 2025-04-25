use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use void_log::log_info;
use crate::orange::{Clan, Track, TrackResult};
use crate::util;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiddleApi {
    my_tag: String,
    my_name: Option<String>,
    opp_tag: String,
    opp_name: Option<String>,
    match_type: Option<String>,
    win_tag: String,
    win_name: Option<String>,
    #[serde(rename = "explain_ch")]
    explain_ch: Option<String>,
    #[serde(rename = "explain_en")]
    explain_en: Option<String>,
    email: Option<String>,
    match_strategy: Option<String>,
    round_score: i64,
    err: bool,
}

impl MiddleApi {
    pub async fn new(tag: &str, is_global: bool) -> reqwest::Result<Self> {
        let body = json!({
            "myTag": tag,
            "isGlobal": is_global,
        });
        let response = Client::new()
            .post("http://cocbzlm.com:8422/api/wardecider")
            .header("isAdmin", "true")
            .json(&body)
            .send()
            .await;
        response?.json().await
    }
    
    pub async fn check_win(&self, pool: &Pool<Postgres>, mut track: Track, is_global: bool, self_tag: &str) -> Track {
        // 格式化对方tag(不战可能反转了my_tag)
        let my_tag = format!("#{}", self.my_tag.replace("#", ""));
        let opp_tag = format!("#{}", self.opp_tag.replace("#", ""));
        let (rival_tag, rival_name, self_name) = if my_tag.eq(self_tag) {
            (opp_tag, self.opp_name.clone(), self.my_name.clone())
        }  else {
            (my_tag, self.my_name.clone(), self.opp_name.clone())
        };

        // 格式化输赢tag
        let win_tag = format!("#{}", self.win_tag.replace("#", ""));
        
        log_info!("rival_tag: {rival_tag} | win_tag: {win_tag} | is_global: {is_global}");

        // 查询对家在数据库记录,没有就新增
        let rival_clan = if let Ok(rc) = Clan::select_tag(pool, &rival_tag, 9, is_global).await {
            log_info!("合作有缓存: {}", rc.name.clone().unwrap());
            rc
        } else {
            let clan = Clan {
                tag: Some(rival_tag.clone()),
                name: rival_name,
                status: Some(9),
                series_id: Some(Uuid::parse_str("4fc2832d-cf1f-47e0-9b54-6c35937c73a4").unwrap()),
                ..Default::default()
            };
            let insert_res = clan.insert(pool).await.unwrap();
            log_info!("新增合作盟: {}", insert_res.rows_affected());
            let opp_clan = Clan::select_tag(pool, &rival_tag, 9, is_global).await;
            opp_clan.unwrap()
        };

        // 组装Track
        track.rival_clan_id = rival_clan.id.unwrap();
        track.rival_tag = rival_clan.tag;
        track.rival_name = rival_clan.name;
        
        track.self_tag = Some(self_tag.to_string());
        track.self_name = self_name;

        // 判断输赢写入Track
        if let Some(rct) = track.rival_tag.as_ref() {
            if win_tag.eq(rct) {
                track.result = TrackResult::Lose;
            } else if self.err {
                track.result = TrackResult::None;
            } else {
                track.result = TrackResult::Win;
            };
        } else { track.result = TrackResult::None; };

        // 返回Track
        track
    }
}

#[tokio::test]
async fn test_win() {
    let pool = util::Config::get().await.get_database().get().await;
    let clan = Clan::select_tag(&pool, "#YL8GUU0Q", 9, true).await.unwrap();
    log_info!("<UNK>: {:?}", clan);
}
