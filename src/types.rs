use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub user_agent: Option<String>,
    pub language: String,
    pub platform: String,
    pub timezone: String,
    pub screen_width: u32,
    pub screen_height: u32,
    pub ms_token: Option<String>,
    pub proxy: Option<String>,
    pub cookies: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct TikTokSession {
    pub config: SessionConfig,
    pub browser: Arc<chromiumoxide::Browser>,
    pub page: Arc<chromiumoxide::Page>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub sec_uid: String,
    pub username: String,
    #[serde(flatten)]
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Video {
    pub id: String,
    pub url: Option<String>,
    #[serde(rename = "createTime")]
    pub create_time: Option<DateTime<Utc>>,
    pub stats: Option<VideoStats>,
    pub author: Option<UserProfile>,
    #[serde(flatten)]
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStats {
    #[serde(rename = "diggCount")]
    pub likes: i64,
    #[serde(rename = "shareCount")]
    pub shares: i64,
    #[serde(rename = "commentCount")]
    pub comments: i64,
    #[serde(rename = "playCount")]
    pub plays: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub text: String,
    pub author: UserProfile,
    #[serde(rename = "diggCount")]
    pub likes_count: i64,
    #[serde(flatten)]
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hashtag {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(flatten)]
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sound {
    pub id: String,
    pub title: Option<String>,
    pub duration: Option<i32>,
    pub original: Option<bool>,
    pub author: Option<UserProfile>,
    #[serde(flatten)]
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestParams {
    pub aid: String,
    pub app_language: String,
    pub app_name: String,
    pub browser_language: String,
    pub browser_name: String,
    pub browser_online: String,
    pub browser_platform: String,
    pub browser_version: String,
    pub channel: String,
    pub cookie_enabled: String,
    pub device_id: String,
    pub device_platform: String,
    pub focus_state: String,
    pub from_page: String,
    pub history_len: String,
    pub is_fullscreen: String,
    pub is_page_visible: String,
    pub os: String,
    pub priority_region: String,
    pub referer: String,
    pub region: String,
    pub screen_height: String,
    pub screen_width: String,
    pub tz_name: String,
    pub webcast_language: String,

    
    // API specific params
    pub sec_uid: Option<String>,
    pub unique_id: Option<String>,
    pub count: Option<String>,
    pub cursor: Option<String>,
    pub aweme_id: Option<String>,
    pub comment_id: Option<String>,
    pub music_id: Option<String>,
    pub challenge_id: Option<String>,
    pub challenge_name: Option<String>,
    pub item_id: Option<String>,
    pub keyword: Option<String>,
    pub web_search_code: Option<String>,

    // Additional params
    #[serde(flatten)]
    pub additional: HashMap<String, String>,
}