use std::time::Duration;

use reqwest::header::{
    ETAG, HeaderMap, HeaderName, HeaderValue, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED,
    USER_AGENT,
};
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
            timeout: Duration::from_secs(10),
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
    Http(String),
    Status(u16),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "HTTP error: {msg}"),
            Self::Status(code) => write!(f, "HTTP status {code}"),
        }
    }
}

impl From<reqwest::Error> for FetchError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(error_chain(&err))
    }
}

fn error_chain(err: &dyn std::error::Error) -> String {
    let mut message = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        let cause_message = cause.to_string();
        if !message.contains(&cause_message) {
            message.push_str(": ");
            message.push_str(&cause_message);
        }
        source = cause.source();
    }
    message
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
            .map_err(FetchError::from)?;

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
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.user_agent)
                .unwrap_or_else(|_| HeaderValue::from_static("rivulet")),
        );

        if let Some(cache) = cache {
            if let Some(etag) = &cache.etag
                && let Ok(value) = HeaderValue::from_str(etag)
            {
                headers.insert(IF_NONE_MATCH, value);
            }
            if let Some(last_modified) = &cache.last_modified
                && let Ok(value) = HeaderValue::from_str(last_modified)
            {
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

        let response = self.client.get(url).headers(headers).send().await?;

        let status = response.status();
        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(std::string::ToString::to_string);
        let last_modified = response
            .headers()
            .get(LAST_MODIFIED)
            .and_then(|value| value.to_str().ok())
            .map(std::string::ToString::to_string);

        if status == StatusCode::NOT_MODIFIED {
            return Ok(FetchResponse {
                body: None,
                etag,
                last_modified,
            });
        }

        if !status.is_success() {
            return Err(FetchError::Status(status.as_u16()));
        }

        let body = response.bytes().await?;
        Ok(FetchResponse {
            body: Some(body),
            etag,
            last_modified,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Layer {
        message: &'static str,
        source: Option<Box<Layer>>,
    }

    impl std::fmt::Display for Layer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(self.message)
        }
    }

    impl std::error::Error for Layer {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            self.source
                .as_deref()
                .map(|s| s as &(dyn std::error::Error + 'static))
        }
    }

    fn layers(messages: &[&'static str]) -> Layer {
        messages
            .iter()
            .rev()
            .fold(None, |source, message| {
                Some(Layer {
                    message,
                    source: source.map(Box::new),
                })
            })
            .expect("at least one layer")
    }

    #[test]
    fn error_chain_joins_all_sources() {
        let error = layers(&[
            "error decoding response body",
            "request or response body error",
            "operation timed out",
        ]);
        assert_eq!(
            error_chain(&error),
            "error decoding response body: request or response body error: operation timed out"
        );
    }

    #[test]
    fn error_chain_skips_sources_already_in_message() {
        let error = layers(&["connect failed: refused", "refused"]);
        assert_eq!(error_chain(&error), "connect failed: refused");
    }

    #[test]
    fn fetch_error_from_reqwest_includes_cause() {
        let error = Client::new()
            .get("not a url")
            .build()
            .expect_err("invalid url");
        let message = FetchError::from(error).to_string();
        assert!(
            message.starts_with("HTTP error: builder error: "),
            "{message}"
        );
        assert!(
            message.len() > "HTTP error: builder error: ".len(),
            "{message}"
        );
    }
}
