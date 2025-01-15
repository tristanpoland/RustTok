use rust_tok::{
    api::{HashtagApi, UserApi, VideoApi}, error::{Result, TikTokError}, TikTokApi, TikTokApiConfig
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the API with custom configuration
    let config = TikTokApiConfig {
        num_sessions: 2,
        headless: true,
        ms_tokens: Some(vec!["your_ms_token".to_string()]),
        ..Default::default()
    };

    // Create API instance
    let api = TikTokApi::new(config).await?;

    // Fetch user information
    let user = api.user_info("therock").await?;
    println!("User info: {:?}", user);

    // Get user's recent videos
    let videos = api.user_videos(&user.sec_uid, 10).await?;
    println!("Found {} videos", videos.len());

    // Get video details
    if let Some(video) = videos.first() {
        let comments = api.video_comments(&video.id, 20).await?;
        println!("Video has {} comments", comments.len());

        // Download video
        let video_bytes = api.video_bytes(&video.id).await?;
        std::fs::write("video.mp4", video_bytes).map_err(|e| TikTokError::from(e.to_string()))?;
    }

    // Search for hashtag and get related videos
    let hashtag = api.hashtag_info("fyp").await?;
    if let Some(hashtag_id) = hashtag.id {
        let trending = api.hashtag_videos(&hashtag_id, 5).await?;
        println!("Found {} trending videos for #fyp", trending.len());
    }

    // Clean up
    api.close().await?;

    Ok(())
}