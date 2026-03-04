use std::time::Duration;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED, ETAG, USER_AGENT};
use reqwest::{Client, StatusCode};

#[derive(Debug, Clone)]
pub struct FetchOptions {
    pub user_agent: String,
    pub timeout: Duration,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            user_agent: "rivulet/0.1".to_string(),
            timeout: Duration::from_secs(15),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheState {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub body: Option<bytes::Bytes>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug)]
pub enum FetchError {
    Http,
    Status,
}

impl From<reqwest::Error> for FetchError {
    fn from(_: reqwest::Error) -> Self {
        Self::Http
    }
}

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    user_agent: String,
}

impl HttpClient {
    pub fn new(options: FetchOptions) -> Result<Self, FetchError> {
        let client = Client::builder()
            .timeout(options.timeout)
            .build()
            .map_err(|_| FetchError::Http)?;

        Ok(Self {
            client,
            user_agent: options.user_agent,
        })
    }

    pub async fn fetch(
        &self,
        url: &str,
        cache: Option<&CacheState>,
        extra_headers: Option<&[(&str, &str)]>,
    ) -> Result<FetchResponse, FetchError> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_str(&self.user_agent).unwrap_or_else(|_| HeaderValue::from_static("rivulet")));

        if let Some(cache) = cache {
            if let Some(etag) = &cache.etag
                && let Ok(value) = HeaderValue::from_str(etag) {
                    headers.insert(IF_NONE_MATCH, value);
                }
            if let Some(last_modified) = &cache.last_modified
                && let Ok(value) = HeaderValue::from_str(last_modified) {
                    headers.insert(IF_MODIFIED_SINCE, value);
                }
        }

        if let Some(extra_headers) = extra_headers {
            for (name, value) in extra_headers {
                if let (Ok(name), Ok(value)) = (
                    HeaderName::from_bytes(name.as_bytes()),
                    HeaderValue::from_str(value),
                ) {
                    headers.insert(name, value);
                }
            }
        }

        let response = self
            .client
            .get(url)
            .headers(headers)
            .send()
            .await?;

        let status = response.status();
        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());
        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.to_string());

        if status == StatusCode::NOT_MODIFIED {
            return Ok(FetchResponse {
                body: None,
                etag,
                last_modified,
            });
        }

        if !status.is_success() {
            return Err(FetchError::Status);
        }

        let body = response.bytes().await?;
        Ok(FetchResponse {
            body: Some(body),
            etag,
            last_modified,
        })
    }
}
