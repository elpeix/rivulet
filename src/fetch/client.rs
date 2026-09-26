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
    pub max_body_bytes: usize,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            user_agent: "rivulet/0.1".to_string(),
            timeout: Duration::from_secs(10),
            max_body_bytes: 10 * 1024 * 1024,
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
    TooLarge(usize),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(msg) => write!(f, "HTTP error: {msg}"),
            Self::Status(code) => write!(f, "HTTP status {code}"),
            Self::TooLarge(limit) => write!(
                f,
                "Feed exceeds the size limit of {} MB",
                limit / (1024 * 1024)
            ),
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
    max_body_bytes: usize,
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
            max_body_bytes: options.max_body_bytes,
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

        let limit = self.max_body_bytes;
        if response
            .content_length()
            .is_some_and(|length| length > limit as u64)
        {
            return Err(FetchError::TooLarge(limit));
        }

        let mut response = response;
        let mut body = bytes::BytesMut::new();
        while let Some(chunk) = response.chunk().await? {
            if body.len() + chunk.len() > limit {
                return Err(FetchError::TooLarge(limit));
            }
            body.extend_from_slice(&chunk);
        }

        Ok(FetchResponse {
            body: Some(body.freeze()),
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

    fn serve_once(headers: &'static str, body_len: usize) -> String {
        use std::io::{Read, Write};

        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = Vec::new();
            let mut buf = [0u8; 1024];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                match stream.read(&mut buf) {
                    Ok(0) | Err(_) => return,
                    Ok(n) => request.extend_from_slice(&buf[..n]),
                }
            }
            let head = format!("HTTP/1.1 200 OK\r\nConnection: close\r\n{headers}\r\n");
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&vec![b'x'; body_len]);
        });
        format!("http://{addr}/feed.xml")
    }

    fn client_with_limit(max_body_bytes: usize) -> HttpClient {
        HttpClient::new(FetchOptions {
            max_body_bytes,
            ..FetchOptions::default()
        })
        .expect("client")
    }

    #[test]
    fn default_max_body_is_ten_megabytes() {
        assert_eq!(FetchOptions::default().max_body_bytes, 10 * 1024 * 1024);
    }

    #[tokio::test]
    async fn fetch_accepts_body_within_limit() {
        let url = serve_once("Content-Length: 100\r\n", 100);
        let response = client_with_limit(100)
            .fetch(&url, None, None)
            .await
            .expect("fetch");
        assert_eq!(response.body.expect("body").len(), 100);
    }

    #[tokio::test]
    async fn fetch_rejects_declared_length_over_limit() {
        let url = serve_once("Content-Length: 101\r\n", 101);
        let error = client_with_limit(100)
            .fetch(&url, None, None)
            .await
            .expect_err("too large");
        assert!(matches!(error, FetchError::TooLarge(100)), "{error}");
    }

    #[tokio::test]
    async fn fetch_rejects_streamed_body_over_limit() {
        let url = serve_once("", 5000);
        let error = client_with_limit(1000)
            .fetch(&url, None, None)
            .await
            .expect_err("too large");
        assert!(matches!(error, FetchError::TooLarge(1000)), "{error}");
    }

    #[test]
    fn too_large_error_shows_limit_in_megabytes() {
        let error = FetchError::TooLarge(10 * 1024 * 1024);
        assert_eq!(error.to_string(), "Feed exceeds the size limit of 10 MB");
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
