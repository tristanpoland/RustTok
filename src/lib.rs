pub mod api;
pub mod error;
pub mod types;

use chromiumoxide::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::target::CreateTargetParams;
use futures::StreamExt;
use rand::{Rng, seq::SliceRandom};
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;
use std::collections::HashMap;
use crate::{
    error::{Result, TikTokError},
    types::{SessionConfig, TikTokSession, RequestParams},
};

impl From<chromiumoxide::error::CdpError> for TikTokError {
    fn from(error: chromiumoxide::error::CdpError) -> Self {
        TikTokError::BrowserError(error.to_string())
    }
}

impl From<url::ParseError> for TikTokError {
    fn from(error: url::ParseError) -> Self {
        TikTokError::InvalidUrl(error.to_string())
    }
}

impl From<String> for TikTokError {
    fn from(error: String) -> Self {
        TikTokError::BrowserError(error)
    }
}

pub struct TikTokApi {
    sessions: Arc<Mutex<Vec<TikTokSession>>>,
    config: TikTokApiConfig,
}

#[derive(Debug, Clone)]
pub struct TikTokApiConfig {
    pub num_sessions: usize,
    pub headless: bool,
    pub ms_tokens: Option<Vec<String>>,
    pub proxies: Option<Vec<String>>,
    pub base_url: String,
    pub browser_args: Option<Vec<String>>,
}

impl Default for TikTokApiConfig {
    fn default() -> Self {
        Self {
            num_sessions: 5,
            headless: true,
            ms_tokens: None,
            proxies: None,
            base_url: "https://www.tiktok.com".to_string(),
            browser_args: None,
        }
    }
}

impl TikTokApi {
    pub async fn new(config: TikTokApiConfig) -> Result<Self> {
        let sessions = Arc::new(Mutex::new(Vec::new()));
        let api = Self { sessions, config };
        api.create_sessions().await?;
        Ok(api)
    }

    pub async fn create_sessions(&self) -> Result<()> {
        let mut sessions = self.sessions.lock().await;

        for _ in 0..self.config.num_sessions {
            let proxy = self.config.proxies
                .as_ref()
                .and_then(|proxies| proxies.choose(&mut rand::thread_rng()))
                .cloned();

            let mut config_builder = BrowserConfig::builder();
            if !self.config.headless {
                config_builder = config_builder.with_head();
            }
            if let Some(proxy_str) = proxy.as_deref() {
                config_builder = config_builder.arg("--proxy-server=".to_string() + proxy_str);
            }
            let browser_config = config_builder.build()?;

            let (browser, mut handler) = Browser::launch(browser_config).await?;
            
            // Handle browser events in background
            tokio::spawn(async move {
                while let Some(event) = handler.next().await {
                    log::debug!("Browser event: {:?}", event);
                }
            });

            let page = browser.new_page(CreateTargetParams::default()).await?;

            let ms_token = self.config.ms_tokens
                .as_ref()
                .and_then(|tokens| tokens.choose(&mut rand::thread_rng()))
                .cloned();

            let config = SessionConfig {
                user_agent: None, // Will be set after page load
                language: "en-US".to_string(),
                platform: "Windows".to_string(),
                timezone: "America/New_York".to_string(),
                screen_width: 1920,
                screen_height: 1080,
                ms_token,
                proxy,
                cookies: None,
            };

            let session = TikTokSession {
                config,
                browser: browser.into(),
                page: page.into(),
            };

            sessions.push(session);
        }

        Ok(())
    }

    pub async fn make_request(
        &self,
        url: &str,
        params: Option<RequestParams>,
        headers: Option<HashMap<String, String>>,
        session_index: Option<usize>,
    ) -> Result<serde_json::Value> {
        let sessions = self.sessions.lock().await;
        let session_idx = session_index.unwrap_or_else(|| {
            rand::thread_rng().gen_range(0..sessions.len())
        });
        
        let session = sessions.get(session_idx)
            .ok_or_else(|| TikTokError::Other(anyhow::anyhow!("Invalid session index")))?;

        let signed_url = self.sign_url(url, &params).await?;
        
        // Prepare fetch request script
        let fetch_script = format!(
            r#"
            async () => {{
                const response = await fetch("{}", {{
                    method: "GET",
                    headers: {},
                }});
                return await response.text();
            }}
            "#,
            signed_url,
            serde_json::to_string(&headers.unwrap_or_default())?
        );

        let result = session.page.evaluate(fetch_script).await?;
        let response_text = result.value().and_then(|v| v.as_str())
            .ok_or_else(|| TikTokError::EmptyResponse)?;

        if response_text.is_empty() {
            return Err(TikTokError::EmptyResponse);
        }

        let json: serde_json::Value = serde_json::from_str(response_text)?;
        
        if let Some(status_code) = json.get("status_code") {
            if status_code != 0 {
                return Err(TikTokError::ApiError(
                    format!("TikTok API error: Status code {}", status_code)
                ));
            }
        }

        Ok(json)
    }

    async fn sign_url(&self, url: &str, params: &Option<RequestParams>) -> Result<String> {
        let sessions = self.sessions.lock().await;
        let session = sessions.get(0)
            .ok_or_else(|| TikTokError::Other(anyhow::anyhow!("No sessions available")))?;

        // Convert params to URL query string if provided
        let mut final_url = if let Some(params) = params {
            let mut url = Url::parse(url)?;
            let param_map = serde_json::to_value(params)?;
            
            if let serde_json::Value::Object(map) = param_map {
                for (key, value) in map {
                    url.query_pairs_mut().append_pair(&key, &value.to_string());
                }
            }
            url.to_string()
        } else {
            url.to_string()
        };

        // Generate X-Bogus
        let script = format!(
            r#"
            async () => {{
                if (typeof window.byted_acrawler === 'undefined') {{
                    return null;
                }}
                return window.byted_acrawler.frontierSign("{}");
            }}
            "#,
            final_url
        );

        let result = session.page.evaluate(script).await?;
        
        let x_bogus = result.value()
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v.get("X-Bogus").and_then(|b| b.as_str()).map(|s| s.to_string()))
            .ok_or(TikTokError::SignatureError)?;

        // Append X-Bogus to URL
        if final_url.contains('?') {
            final_url.push('&');
        } else {
            final_url.push('?');
        }
        final_url.push_str(&format!("X-Bogus={}", x_bogus));

        Ok(final_url)
    }

    pub async fn close(&self) -> Result<()> {
        let mut sessions = self.sessions.lock().await;
        for session in sessions.iter_mut() {
            <chromiumoxide::Page as Clone>::clone(&session.page).close().await?;
            Arc::try_unwrap(session.browser.clone()).unwrap().close().await?;
        }
        sessions.clear();
        Ok(())
    }
}