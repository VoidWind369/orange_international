use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MiddleReadCompo {
    #[serde(rename(deserialize = "minTHAvg"))]
    #[serde(deserialize_with = "string_to_f32")]
    min_th_avg: f32,
    #[serde(rename(deserialize = "maxTHAvg"))]
    #[serde(deserialize_with = "string_to_f32")]
    max_th_avg: f32,
    calculated_time: String,
    calculated_composition: Vec<String>,
    global: bool,
}

fn string_to_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<f32>().map_err(serde::de::Error::custom)
}

impl MiddleReadCompo {
    pub async fn get() -> Self {
        let response = reqwest::get("http://cocbzlm.com:8422/api/accinfo/readCompo")
            .await
            .unwrap();
        response.json().await.unwrap()
    }

    async fn get_text() -> String {
        let response = reqwest::get("http://cocbzlm.com:8422/api/accinfo/readCompo")
            .await
            .unwrap();
        response.text().await.unwrap()
    }
}

#[tokio::test]
async fn get_test() {
    use void_log::*;
    let m = MiddleReadCompo::get().await;
    log_info!("{:?}", m)
}
