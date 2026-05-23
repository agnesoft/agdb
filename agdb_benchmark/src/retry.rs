use crate::bench_error::BenchError;
use crate::bench_result::BenchResult;
use crate::config::RetryConfig;
use std::time::Duration;

pub(crate) struct RetryState {
    consecutive_failures: u32,
}

impl RetryState {
    pub(crate) fn new() -> Self {
        Self {
            consecutive_failures: 0,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.consecutive_failures = 0;
    }

    pub(crate) async fn on_failure(
        &mut self,
        retry: &RetryConfig,
        context: &str,
        cause: &str,
    ) -> BenchResult<()> {
        self.consecutive_failures += 1;

        if self.consecutive_failures > retry.max_consecutive_failures {
            return Err(BenchError {
                description: format!(
                    "{context}: too many consecutive failures ({}) - last error: {cause}",
                    self.consecutive_failures
                ),
            });
        }

        tokio::time::sleep(backoff_delay(retry, self.consecutive_failures)).await;
        Ok(())
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
