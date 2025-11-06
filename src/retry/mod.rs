use std::time::Duration;
use tokio::time::sleep;
use tracing::warn;

/// Retry configuration
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

/// Execute a function with exponential backoff retry
pub async fn with_retry<F, Fut, T, E>(
    config: &RetryConfig,
    operation_name: &str,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay;
    let mut attempt = 1;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= config.max_attempts => {
                warn!(
                    "Operation '{}' failed after {} attempts: {}",
                    operation_name, config.max_attempts, e
                );
                return Err(e);
            }
            Err(e) => {
                warn!(
                    "Operation '{}' failed (attempt {}/{}): {}. Retrying in {:?}...",
                    operation_name, attempt, config.max_attempts, e, delay
                );

                sleep(delay).await;

                // Exponential backoff
                delay = std::cmp::min(
                    Duration::from_secs_f64(delay.as_secs_f64() * config.multiplier),
                    config.max_delay,
                );

                attempt += 1;
            }
        }
    }
}
