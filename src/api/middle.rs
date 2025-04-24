use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use void_log::log_info;
use crate::orange::{Clan, Track, TrackResult};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiddleApi {
    my_tag: Option<String>,
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
    
    pub async fn check_win(&self, pool: &Pool<Postgres>, mut track: Track, is_global: bool) -> Track {
        // 格式化对方tag
        let opp_tag = format!("#{}", self.opp_tag.replace("#", ""));

        // 格式化输赢tag
        let win_tag = format!("#{}", self.win_tag.replace("#", ""));

        // 查询对家在数据库记录,没有就新增
        let opp_clan = if let Ok(oc) = Clan::select_tag(pool, &opp_tag, 9, is_global).await {
            oc
        } else {
            let clan = Clan {
                tag: Some(self.opp_tag.clone()),
                name: self.opp_name.clone(),
                status: Some(9),
                series_id: Some(Uuid::parse_str("4fc2832d-cf1f-47e0-9b54-6c35937c73a4").unwrap()),
                ..Default::default()
            };
            let insert_res = clan.insert(pool).await.unwrap();
            log_info!("新增合作盟: {}", insert_res.rows_affected());
            let opp_clan = Clan::select_tag(pool, &opp_tag, 9, is_global).await;
            opp_clan.unwrap()
        };

        // 组装Track
        track.rival_clan_id = opp_clan.id.unwrap();
        track.rival_tag = opp_clan.tag;
        track.rival_name = opp_clan.name;

        // 判断输赢写入Track
        if let Some(rct) = track.rival_tag.as_ref() {
            if win_tag.eq(rct) {
                track.result = TrackResult::Lose;
            } else {
                track.result = TrackResult::Win;
                track.self_tag = Option::from(win_tag);
            };
        } else { track.result = TrackResult::None; };

        // 返回Track
        track
    }
}
