use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use crate::orange::Track;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MiddleApi {
    my_tag: String,
    my_name: String,
    opp_tag: String,
    opp_name: String,
    match_type: Value,
    win_tag: String,
    win_name: String,
    explain_ch: Option<String>,
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
            .post("https://api.middleinity.app/api")
            .header("isAdmin", "true")
            .json(&body)
            .send()
            .await;
        response?.json().await
    }
    
    pub fn check_win(&self, self_tag:&str) -> Track {
        let my_tag = format!("#{}", self.my_tag.replace("#", ""));
        let ck = my_tag.eq_ignore_ascii_case(self_tag);
        if ck {
            Track {
                id: Default::default(),
                self_clan_id: Default::default(),
                rival_clan_id: Default::default(),
                self_history_point: 0,
                rival_history_point: 0,
                create_time: Default::default(),
                self_now_point: 0,
                rival_now_point: 0,
                round_id: Default::default(),
                result: Default::default(),
                round_code: None,
                self_tag: None,
                self_name: None,
                rival_tag: None,
                rival_name: None,
            }
        } else { 
            Track::default()
        }
    }
}
