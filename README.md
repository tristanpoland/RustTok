# WARNING: This Crate is in-development
Nothing is yet guarunteed to work correctly

# TikTok-RS

An unofficial Rust library for interacting with TikTok's web API. This library provides a safe, fast, and feature-rich way to interact with TikTok programmatically.

## Features

- Full API coverage (users, videos, hashtags, sounds, etc.)
- Async/await support with Tokio
- Built-in browser automation with stealth mode
- Automatic session management and rotation
- Strong type system and comprehensive error handling
- Thread-safe and memory-safe by design
- Extensive documentation and examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tiktok-rs = "0.1.0"
```

## Quick Start

```rust
use tiktok_rs::{TikTokApi, TikTokApiConfig, api::UserApi};

#[tokio::main]
async fn main() -> Result<()> {
    // Create API instance
    let api = TikTokApi::new(TikTokApiConfig::default()).await?;
    
    // Get user info
    let user = api.user_info("therock").await?;
    println!("Found user: {}", user.username);
    
    // Get recent videos
    let videos = api.user_videos(&user.sec_uid, 10).await?;
    println!("Found {} videos", videos.len());
    
    // Clean up
    api.close().await?;
    Ok(())
}
```

## Advanced Usage

### Session Configuration

```rust
let config = TikTokApiConfig {
    num_sessions: 5,
    headless: true,
    ms_tokens: Some(vec!["your_ms_token".to_string()]),
    proxies: Some(vec!["http://proxy1.com", "http://proxy2.com"]),
    ..Default::default()
};
```

### Download Videos

```rust
// Get video bytes
let video_bytes = api.video_bytes("video_id").await?;

// Save to file
std::fs::write("video.mp4", video_bytes)?;
```

### Search and Trending

```rust
// Search for users
let users = api.search_users("dance", 10).await?;

// Get trending videos
let trending = api.trending_videos(20).await?;
```

## Error Handling

The library uses custom error types for better error handling:

```rust
match api.user_info("nonexistent").await {
    Ok(user) => println!("Found user: {}", user.username),
    Err(TikTokError::NotFound) => println!("User not found"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This is an unofficial library and is not affiliated with, authorized, maintained, sponsored or endorsed by TikTok or any of its affiliates or subsidiaries. Use at your own risk.
