use async_trait::async_trait;

use crate::{
    error::{Result, TikTokError},
    types::{Sound, Video, RequestParams},
    TikTokApi,
};

#[async_trait]
pub trait SoundApi {
    async fn sound_info(&self, sound_id: &str) -> Result<Sound>;
    async fn sound_videos(&self, sound_id: &str, count: usize) -> Result<Vec<Video>>;
}

#[async_trait]
impl SoundApi for TikTokApi {
    async fn sound_info(&self, sound_id: &str) -> Result<Sound> {
        let params = RequestParams {
            music_id: Some(sound_id.to_string()),
            ..Default::default()
        };

        let response = self.make_request(
            "https://www.tiktok.com/api/music/detail/",
            Some(params),
            None,
            None
        ).await?;

        if let Some(music_info) = response.get("musicInfo") {
            serde_json::from_value(music_info.clone())
                .map_err(|e| e.into())
        } else {
            Err(TikTokError::NotFound)
        }
    }

    async fn sound_videos(&self, sound_id: &str, count: usize) -> Result<Vec<Video>> {
        let mut videos = Vec::new();
        let mut cursor = 0;

        while videos.len() < count {
            let params = RequestParams {
                music_id: Some(sound_id.to_string()),
                count: Some(30.min(count - videos.len()).to_string()),
                cursor: Some(cursor.to_string()),
                ..Default::default()
            };

            let response = self.make_request(
                "https://www.tiktok.com/api/music/item_list/",
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