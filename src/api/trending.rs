use async_trait::async_trait;

use crate::{
    error::Result,
    types::{Video, RequestParams},
    TikTokApi,
};

#[async_trait]
pub trait TrendingApi {
    async fn trending_videos(&self, count: usize) -> Result<Vec<Video>>;
}

#[async_trait]
impl TrendingApi for TikTokApi {
    async fn trending_videos(&self, count: usize) -> Result<Vec<Video>> {
        let mut videos = Vec::new();

        while videos.len() < count {
            let params = RequestParams {
                from_page: "fyp".to_string(),
                count: Some(count.to_string()),
                ..Default::default()
            };

            let response = self.make_request(
                "https://www.tiktok.com/api/recommend/item_list/",
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
        }

        Ok(videos)
    }
}