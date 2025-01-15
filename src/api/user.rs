use async_trait::async_trait;

use crate::{
    error::Result,
    types::{UserProfile, Video, RequestParams},
    TikTokApi,
};

#[async_trait]
pub trait UserApi {
    async fn user_info(&self, username: &str) -> Result<UserProfile>;
    async fn user_videos(&self, sec_uid: &str, count: usize) -> Result<Vec<Video>>;
    async fn user_liked_videos(&self, sec_uid: &str, count: usize) -> Result<Vec<Video>>;
}

#[async_trait]
impl UserApi for TikTokApi {
    async fn user_info(&self, username: &str) -> Result<UserProfile> {
        let params = RequestParams {
            sec_uid: Some("".to_string()),
            unique_id: Some(username.to_string()),
            ..Default::default()
        };

        let response = self.make_request(
            "https://www.tiktok.com/api/user/detail/",
            Some(params),
            None,
            None,
        ).await?;

        serde_json::from_value(response["userInfo"].clone())
            .map_err(|e| e.into())
    }

    async fn user_videos(&self, sec_uid: &str, count: usize) -> Result<Vec<Video>> {
        let mut videos = Vec::new();
        let mut cursor = 0;
        
        while videos.len() < count {
            let params = RequestParams {
                sec_uid: Some(sec_uid.to_string()),
                count: Some(35.min(count - videos.len()).to_string()),
                cursor: Some(cursor.to_string()),
                ..Default::default()
            };

            let response = self.make_request(
                "https://www.tiktok.com/api/post/item_list/",
                Some(params),
                None,
                None,
            ).await?;

            let items = response["itemList"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

            for item in items {
                videos.push(serde_json::from_value(item.clone())?);
                if videos.len() >= count {
                    break;
                }
            }

            if !response["hasMore"].as_bool().unwrap_or(false) {
                break;
            }

            cursor = response["cursor"]
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("Invalid cursor format"))?;
        }

        Ok(videos)
    }

    async fn user_liked_videos(&self, sec_uid: &str, count: usize) -> Result<Vec<Video>> {
        let mut videos = Vec::new();
        let mut cursor = 0;
        
        while videos.len() < count {
            let params = RequestParams {
                sec_uid: Some(sec_uid.to_string()),
                count: Some(35.min(count - videos.len()).to_string()),
                cursor: Some(cursor.to_string()),
                ..Default::default()
            };

            let response = self.make_request(
                "https://www.tiktok.com/api/favorite/item_list",
                Some(params),
                None,
                None,
            ).await?;

            let items = response["itemList"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

            for item in items {
                videos.push(serde_json::from_value(item.clone())?);
                if videos.len() >= count {
                    break;
                }
            }

            if !response["hasMore"].as_bool().unwrap_or(false) {
                break;
            }

            cursor = response["cursor"]
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("Invalid cursor format"))?;
        }

        Ok(videos)
    }
}