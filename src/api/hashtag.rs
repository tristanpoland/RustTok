use async_trait::async_trait;

use crate::{
    error::{Result, TikTokError},
    types::{Hashtag, Video, RequestParams},
    TikTokApi,
};

#[async_trait]
pub trait HashtagApi {
    async fn hashtag_info(&self, name: &str) -> Result<Hashtag>;
    async fn hashtag_videos(&self, hashtag_id: &str, count: usize) -> Result<Vec<Video>>;
}

#[async_trait]
impl HashtagApi for TikTokApi {
    async fn hashtag_info(&self, name: &str) -> Result<Hashtag> {
        let params = RequestParams {
            challenge_name: Some(name.to_string()),
            ..Default::default()
        };

        let response = self.make_request(
            "https://www.tiktok.com/api/challenge/detail/",
            Some(params),
            None,
            None
        ).await?;

        let challenge_info = response["challengeInfo"].clone();
        if challenge_info.is_null() {
            return Err(TikTokError::NotFound);
        }

        serde_json::from_value(challenge_info)
            .map_err(|e| e.into())
    }

    async fn hashtag_videos(&self, hashtag_id: &str, count: usize) -> Result<Vec<Video>> {
        let mut videos = Vec::new();
        let mut cursor = 0;

        while videos.len() < count {
            let params = RequestParams {
                challenge_id: Some(hashtag_id.to_string()),
                count: Some(35.min(count - videos.len()).to_string()),
                cursor: Some(cursor.to_string()),
                ..Default::default()
            };

            let response = self.make_request(
                "https://www.tiktok.com/api/challenge/item_list/",
                Some(params),
                None,
                None
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