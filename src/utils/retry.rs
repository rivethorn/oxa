use std::time::Duration;
use tokio::time::sleep;
use utils::error::AppError;

/// Retry with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    E: std::fmt::Display,
{
    let mut delay = initial_delay;
    
    for attempt in 1..=max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_attempts => {
                eprintln!("Attempt {} failed: {}. Retrying in {}s...", attempt, e, delay.as_secs());
                sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
    
    unreachable!()
}

/// Check if an error is likely retryable
pub fn is_retryable_error(error: &AppError) -> bool {
    error.is_retryable()
}

/// Maximum retry attempts for network operations
pub const MAX_RETRIES: u32 = 3;

/// Initial delay for retry logic (in seconds)
pub const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);