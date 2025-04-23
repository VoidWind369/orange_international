use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
}
