use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use toasty::Model;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Default, Model, Serialize, Deserialize)]
pub struct Series {
    #[key]
    #[auto(uuid(v4))]
    id: Uuid,
    name: Option<String>,
    #[serde(skip_deserializing)]
    create_time: Timestamp,
    #[serde(skip_deserializing)]
    #[update(jiff::Timestamp::now())]
    update_time: Timestamp,
    status: Option<i16>,
    #[auto(uuid(v4))]
    code: Uuid,
}

impl Series {
    async fn db() -> toasty::Db {
        toasty::Db::builder()
            .models(toasty::models!(crate::*))
            .connect("postgresql://orange_international:7rpE7EYnZWFRSTzm@1.orgvoid.top:37700/orange_international")
            .await
            .unwrap()
    }

    async fn selects() -> Vec<Series> {
        let mut db = Self::db().await;
        Series::all().exec(&mut db).await.unwrap()
    }

    async fn insert(&self) -> Series {
        let mut db = Self::db().await;
        Self::create()
            .name(&self.name)
            .create_time(Timestamp::now())
            .status(Some(1))
            .code(self.code)
            .exec(&mut db)
            .await
            .unwrap()
    }
}
