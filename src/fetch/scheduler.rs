use std::sync::Arc;

use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use super::client::{CacheState, FetchError, FetchResponse, HttpClient};

#[derive(Debug, Clone)]
pub struct FetchJob {
    pub feed_id: i64,
    pub url: String,
    pub cache: Option<CacheState>,
}

#[derive(Debug, Clone)]
pub struct FetchResult {
    pub body: Option<bytes::Bytes>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Debug)]
pub enum SchedulerError {
    Fetch,
    Acquire,
}

impl From<FetchError> for SchedulerError {
    fn from(_: FetchError) -> Self {
        Self::Fetch
    }
}

impl From<tokio::sync::AcquireError> for SchedulerError {
    fn from(_: tokio::sync::AcquireError) -> Self {
        Self::Acquire
    }
}

pub struct Scheduler {
    client: HttpClient,
    semaphore: Arc<Semaphore>,
}

impl Scheduler {
    pub fn new(client: HttpClient, max_concurrency: usize) -> Self {
        Self {
            client,
            semaphore: Arc::new(Semaphore::new(max_concurrency)),
        }
    }

    pub async fn run(
        &self,
        jobs: Vec<FetchJob>,
    ) -> Vec<(FetchJob, Result<FetchResult, SchedulerError>)> {
        let mut set = JoinSet::new();

        for job in jobs {
            let client = self.client.clone();
            let semaphore = Arc::clone(&self.semaphore);
            set.spawn(async move {
                let permit = semaphore.acquire_owned().await.map_err(SchedulerError::from);
                if let Err(error) = permit {
                    return (job, Err(error));
                }

                match client
                    .fetch(job.url.as_str(), job.cache.as_ref(), None)
                    .await
                    .map_err(SchedulerError::from)
                {
                    Ok(response) => {
                        let mapped = map_response(&job, response);
                        (job, Ok(mapped))
                    }
                    Err(error) => (job, Err(error)),
                }
            });
        }

        let mut results = Vec::new();
        while let Some(joined) = set.join_next().await {
            if let Ok(result) = joined {
                results.push(result);
            }
        }

        results
    }
}

fn map_response(_job: &FetchJob, response: FetchResponse) -> FetchResult {
    FetchResult {
        body: response.body,
        etag: response.etag,
        last_modified: response.last_modified,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::client::{FetchOptions, HttpClient};

    fn test_client() -> HttpClient {
        HttpClient::new(FetchOptions {
            user_agent: "test/1.0".to_string(),
            timeout: std::time::Duration::from_secs(5),
        })
        .unwrap()
    }

    #[tokio::test]
    async fn scheduler_handles_invalid_urls() {
        let client = test_client();
        let scheduler = Scheduler::new(client, 2);
        let jobs = vec![
            FetchJob {
                feed_id: 1,
                url: "http://invalid.test.localhost/feed.xml".to_string(),
                cache: None,
            },
            FetchJob {
                feed_id: 2,
                url: "http://invalid.test.localhost/other.xml".to_string(),
                cache: None,
            },
        ];

        let results = scheduler.run(jobs).await;
        assert_eq!(results.len(), 2);
        for (_job, result) in &results {
            assert!(result.is_err());
        }
    }

    #[tokio::test]
    async fn scheduler_empty_jobs() {
        let client = test_client();
        let scheduler = Scheduler::new(client, 4);
        let results = scheduler.run(vec![]).await;
        assert!(results.is_empty());
    }

    #[test]
    fn map_response_copies_fields() {
        let job = FetchJob {
            feed_id: 1,
            url: "http://example.com".to_string(),
            cache: None,
        };
        let response = FetchResponse {
            body: Some(bytes::Bytes::from("hello")),
            etag: Some("etag-1".to_string()),
            last_modified: Some("Mon, 01 Jan 2024".to_string()),
        };
        let result = map_response(&job, response);
        assert_eq!(result.body.as_ref().map(|b| b.as_ref()), Some(b"hello".as_ref()));
        assert_eq!(result.etag.as_deref(), Some("etag-1"));
        assert_eq!(result.last_modified.as_deref(), Some("Mon, 01 Jan 2024"));
    }
}
