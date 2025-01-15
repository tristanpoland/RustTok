use async_trait::async_trait;

use crate::{
    types::{UserProfile, Video, RequestParams},
    error::Result,
    TikTokApi,
};

#[async_trait]
pub trait SearchApi {
    async fn search_users(&self, query: &str, count: usize) -> Result<Vec<UserProfile>>;
    async fn search_videos(&self, query: &str, count: usize) -> Result<Vec<Video>>;
    async fn search_type(&self, query: &str, obj_type: SearchType, count: usize) -> Result<Vec<serde_json::Value>>;
}

#[derive(Debug, Clone, Copy)]
pub enum SearchType {
    User,
    Video,
    // Can add more search types as TikTok supports them
}

#[async_trait]
impl SearchApi for TikTokApi {
    async fn search_users(&self, query: &str, count: usize) -> Result<Vec<UserProfile>> {
        let results = self.search_type(query, SearchType::User, count).await?;
        
        let users = results.into_iter()
            .filter_map(|item| {
                let user_info = item.get("user_info")?;
                serde_json::from_value(user_info.clone()).ok()
            })
            .collect();

        Ok(users)
    }

    async fn search_videos(&self, query: &str, count: usize) -> Result<Vec<Video>> {
        let results = self.search_type(query, SearchType::Video, count).await?;
        
        let videos = results.into_iter()
            .filter_map(|item| {
                serde_json::from_value(item).ok()
            })
            .collect();

        Ok(videos)
    }

    async fn search_type(&self, query: &str, obj_type: SearchType, count: usize) -> Result<Vec<serde_json::Value>> {
        let mut results = Vec::new();
        let mut cursor = 0;

        let type_path = match obj_type {
            SearchType::User => "user",
            SearchType::Video => "video",
        };

        while results.len() < count {
            let params = RequestParams {
                keyword: Some(query.to_string()),
                cursor: Some(cursor.to_string()),
                from_page: "search".to_string(),
                web_search_code: Some(r#"{"tiktok":{"client_params_x":{"search_engine":{"ies_mt_user_live_video_card_use_libra":1,"mt_search_general_user_live_card":1}},"search_server":{}}}"#.to_string()),
                ..Default::default()
            };

            let response = self.make_request(
                &format!("https://www.tiktok.com/api/search/{}/full/", type_path),
                Some(params),
                None,
                None
            ).await?;

            let items = match obj_type {
                SearchType::User => response["user_list"].as_array(),
                SearchType::Video => response["item_list"].as_array(),
            }.ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

            for item in items {
                results.push(item.clone());
                if results.len() >= count {
                    break;
                }
            }

            if !response["has_more"].as_bool().unwrap_or(false) {
                break;
            }

            cursor = response["cursor"]
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("Invalid cursor format"))?;
        }

        Ok(results)
    }
}