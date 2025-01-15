use async_trait::async_trait;
use reqwest::Url;

use crate::{
    error::{Result, TikTokError},
    types::{Video, Comment, RequestParams},
    TikTokApi,
};

#[async_trait]
pub trait VideoApi {
    async fn video_info(&self, video_id: &str) -> Result<Video>;
    async fn video_bytes(&self, video_id: &str) -> Result<Vec<u8>>;
    async fn video_comments(&self, video_id: &str, count: usize) -> Result<Vec<Comment>>;
    async fn video_by_url(&self, url: &str) -> Result<Video>;
    async fn related_videos(&self, video_id: &str, count: usize) -> Result<Vec<Video>>;
}

#[async_trait]
impl VideoApi for TikTokApi {
    async fn video_info(&self, video_id: &str) -> Result<Video> {
        let params = RequestParams {
            aweme_id: Some(video_id.to_string()),
            ..Default::default()
        };

        let response = self.make_request(
            "https://www.tiktok.com/api/item/detail/",
            Some(params),
            None,
            None
        ).await?;

        let item_info = response["itemInfo"]["itemStruct"].clone();
        serde_json::from_value(item_info)
            .map_err(|e| e.into())
    }

    async fn video_bytes(&self, video_id: &str) -> Result<Vec<u8>> {
        // First get video info to get download URL
        let video = self.video_info(video_id).await?;
        
        let download_url = video.raw_data["video"]["downloadAddr"]
            .as_str()
            .ok_or_else(|| TikTokError::Other(anyhow::anyhow!("No download URL found")))?;
        
        // Prepare headers
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("range", "bytes=0-".parse().unwrap());
        headers.insert("accept-encoding", "identity;q=1, *;q=0".parse().unwrap());
        headers.insert("referer", "https://www.tiktok.com/".parse().unwrap());
        
        // Build client
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()?;

        // Make request
        let response = client
            .get(download_url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(TikTokError::RequestError(response.error_for_status().unwrap_err()));
        }

        Ok(response.bytes().await?.to_vec())
    }

    async fn video_comments(&self, video_id: &str, count: usize) -> Result<Vec<Comment>> {
        let mut comments = Vec::new();
        let mut cursor = 0;

        while comments.len() < count {
            let params = RequestParams {
                aweme_id: Some(video_id.to_string()),
                count: Some(20.min(count - comments.len()).to_string()),
                cursor: Some(cursor.to_string()),
                ..Default::default()
            };

            let response = self.make_request(
                "https://www.tiktok.com/api/comment/list/",
                Some(params),
                None,
                None
            ).await?;

            let items = response["comments"]
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

            for item in items {
                comments.push(serde_json::from_value(item.clone())?);
                if comments.len() >= count {
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

        Ok(comments)
    }

    async fn video_by_url(&self, url: &str) -> Result<Video> {
        let video_id = extract_video_id_from_url(url)?;
        self.video_info(&video_id).await
    }

    async fn related_videos(&self, video_id: &str, count: usize) -> Result<Vec<Video>> {
        let mut videos = Vec::new();
        
        let params = RequestParams {
            aweme_id: Some(video_id.to_string()),
            count: Some(count.to_string()),
            ..Default::default()
        };

        let response = self.make_request(
            "https://www.tiktok.com/api/related/item_list/",
            Some(params),
            None,
            None
        ).await?;

        let items = response["itemList"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid response format"))?;

        for item in items.iter().take(count) {
            videos.push(serde_json::from_value(item.clone())?);
        }

        Ok(videos)
    }
}

fn extract_video_id_from_url(url: &str) -> Result<String> {
    let parsed_url = Url::parse(url)?;
    let path_segments: Vec<_> = parsed_url.path_segments()
        .ok_or_else(|| TikTokError::InvalidUrl("No path segments found".into()))?
        .collect();

    // Check format "@username/video/12345"
    if path_segments.len() >= 3 && path_segments[1] == "video" {
        Ok(path_segments[2].to_string())
    } else {
        Err(TikTokError::InvalidUrl(
            "URL format not supported. Example of supported URL: https://www.tiktok.com/@username/video/12345".into()
        ))
    }
}