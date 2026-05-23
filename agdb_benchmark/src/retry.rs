use crate::bench_error::BenchError;
use crate::bench_result::BenchResult;
use crate::config::RetryConfig;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub(crate) async fn with_retry<O, S, F>(
    retry: &RetryConfig,
    context: &str,
    state: &mut S,
    mut operation: F,
) -> BenchResult<O>
where
    F: for<'a> FnMut(&'a mut S) -> Pin<Box<dyn Future<Output = BenchResult<O>> + Send + 'a>>,
{
    let mut consecutive_failures = 0u32;

    loop {
        match operation(state).await {
            Ok(value) => return Ok(value),
            Err(error) => {
                consecutive_failures += 1;

                if consecutive_failures > retry.max_consecutive_failures {
                    return Err(BenchError {
                        description: format!(
                            "{context}: too many consecutive failures ({consecutive_failures}) - last error: {}",
                            error.description
                        ),
                    });
                }

                tokio::time::sleep(backoff_delay(retry, consecutive_failures)).await;
            }
        }
    }
}

fn backoff_delay(retry: &RetryConfig, consecutive_failures: u32) -> Duration {
    let power = consecutive_failures.saturating_sub(1).min(20);
    let base = retry.base_delay_ms.max(1);
    let max = retry.max_delay_ms.max(base);
    let exponential = base.saturating_mul(1u64 << power);
    let capped = exponential.min(max);

    // Add small jitter to avoid synchronized retry bursts from many workers.
    let jitter = if capped < 4 {
        0
    } else {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64 % (capped / 4))
            .unwrap_or(0)
    };

    Duration::from_millis(capped + jitter)
}
