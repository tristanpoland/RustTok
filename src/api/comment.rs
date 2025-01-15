use async_trait::async_trait;

use crate::{
    error::{Result, TikTokError},
    types::{Comment, RequestParams},
    TikTokApi,
};

#[async_trait]
pub trait CommentApi {
    async fn comment_info(&self, comment_id: &str) -> Result<Comment>;
    async fn comment_replies(&self, comment_id: &str, count: usize) -> Result<Vec<Comment>>;
}

#[async_trait]
impl CommentApi for TikTokApi {
    async fn comment_info(&self, comment_id: &str) -> Result<Comment> {
        let params = RequestParams {
            comment_id: Some(comment_id.to_string()),
            ..Default::default()
        };

        let response = self.make_request(
            "https://www.tiktok.com/api/comment/detail/",
            Some(params),
            None,
            None
        ).await?;

        if let Some(comment_data) = response.get("comment") {
            serde_json::from_value(comment_data.clone())
                .map_err(|e| e.into())
        } else {
            Err(TikTokError::NotFound)
        }
    }

    async fn comment_replies(&self, comment_id: &str, count: usize) -> Result<Vec<Comment>> {
        let mut replies = Vec::new();
        let mut cursor = 0;

        while replies.len() < count {
            let params = RequestParams {
                comment_id: Some(comment_id.to_string()),
                count: Some(20.min(count - replies.len()).to_string()),
                cursor: Some(cursor.to_string()),
                ..Default::default()
            };

            let response = self.make_request(
                "https://www.tiktok.com/api/comment/list/reply/",
                Some(params),
                None,
                None
            ).await?;

            let items = response["comments"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

            for item in items {
                replies.push(serde_json::from_value(item.clone())?);
                if replies.len() >= count {
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

        Ok(replies)
    }
}