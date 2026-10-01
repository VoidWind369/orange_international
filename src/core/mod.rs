use serde::{Deserialize, Serialize};

use crate::orange::Track;

pub mod registration;
pub mod reverse;

/// # Api返回格式
pub enum RegResponse {
    Successful(Track),
    NotRound,
    TagNotFound,
    ClanNotFound(ClanTag),
    ClanNotOwned(ClanTag),
}

/// # 传入的登记信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackReg {
    self_tag: String,
    rival_tag: Option<String>,
    last: Option<bool>,
    is_global: Option<bool>,
}

/// # 部落标签与服务器信息
pub struct ClanTag {
    pub tag: String,
    pub is_global: bool,
}
