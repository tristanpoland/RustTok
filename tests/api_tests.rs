use rust_tok::{
    api::{HashtagApi, UserApi, VideoApi}, error::{Result, TikTokError}, TikTokApi, TikTokApiConfig
};

#[tokio::test]
async fn test_user_info() -> Result<()> {
    let api = TikTokApi::new(TikTokApiConfig::default()).await?;
    
    let user = api.user_info("therock").await?;
    assert_eq!(user.username, "therock");
    assert!(!user.sec_uid.is_empty());
    
    api.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_video_info() -> Result<()> {
    let api = TikTokApi::new(TikTokApiConfig::default()).await?;
    
    // Use a known video ID
    let video = api.video_info("7041997751718137094").await?;
    assert!(!video.id.is_empty());
    assert!(video.create_time.is_some());
    
    api.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_hashtag_videos() -> Result<()> {
    let api = TikTokApi::new(TikTokApiConfig::default()).await?;
    
    let hashtag = api.hashtag_info("fyp").await?;
    assert!(hashtag.id.is_some());
    
    let videos = api.hashtag_videos(hashtag.id.as_ref().unwrap(), 5).await?;
    assert_eq!(videos.len(), 5);
    
    api.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<()> {
    let api = TikTokApi::new(TikTokApiConfig::default()).await?;
    
    match api.user_info("this_user_definitely_does_not_exist_12345").await {
        Err(TikTokError::NotFound) => (),
        _ => panic!("Expected NotFound error"),
    }
    
    api.close().await?;
    Ok(())
}

// Test utilities
mod util {
    use serde_json::json;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    pub async fn setup_mock_server() -> MockServer {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/api/user/detail/"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "userInfo": {
                        "user": {
                            "id": "123",
                            "uniqueId": "test_user",
                            "secUid": "sec123"
                        }
                    }
                })))
            .mount(&mock_server)
            .await;
            
        mock_server
    }
}

#[tokio::test]
async fn test_with_mocks() -> Result<()> {
    let mock_server = util::setup_mock_server().await;
    
    let config = TikTokApiConfig {
        base_url: mock_server.uri(),
        ..Default::default()
    };
    
    let api = TikTokApi::new(config).await?;
    let user = api.user_info("test_user").await?;
    
    assert_eq!(user.username, "test_user");
    assert_eq!(user.user_id, "123");
    
    api.close().await?;
    Ok(())
}