use crate::bench_error::BenchError;
use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::database::admin_api;
use crate::results::MemoryStats;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use reqwest::Client;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub(crate) struct ServerMemoryMonitor {
    api: AgdbApi<ReqwestClient>,
    samples: Arc<Mutex<Vec<u64>>>,
    end_delay: Duration,
    stop_sender: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl ServerMemoryMonitor {
    pub(crate) async fn start(
        config: &Config,
        address: &str,
        client: &Client,
    ) -> BenchResult<Self> {
        let mut api = admin_api(client.clone(), address);
        api.user_login("admin", "admin").await?;

        let start_memory = api.admin_status().await?.1.memory;
        let samples = Arc::new(Mutex::new(vec![start_memory]));
        let task_samples = samples.clone();
        let interval = Duration::from_millis(config.server.memory_poll_interval_ms.max(1));
        let end_delay = Duration::from_millis(config.server.memory_end_delay_ms.max(1));
        let retry = config.server.retry.max_consecutive_failures;
        let (stop_sender, mut stop_receiver) = watch::channel(false);

        let address = address.to_string();
        let client = client.clone();

        let task = tokio::spawn(async move {
            let mut failures = 0u32;
            let mut task_api = admin_api(client, &address);
            if task_api.user_login("admin", "admin").await.is_err() {
                return;
            }

            loop {
                tokio::select! {
                    _ = stop_receiver.changed() => {
                        if *stop_receiver.borrow() {
                            break;
                        }
                    }
                    _ = tokio::time::sleep(interval) => {
                        match task_api.admin_status().await {
                            Ok((_, status)) => {
                                failures = 0;
                                if let Ok(mut guard) = task_samples.lock() {
                                    guard.push(status.memory);
                                }
                            }
                            Err(_) => {
                                failures = failures.saturating_add(1);
                                if failures > retry {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            api,
            samples,
            end_delay,
            stop_sender,
            task,
        })
    }

    pub(crate) async fn finish(self) -> BenchResult<MemoryStats> {
        let Self {
            api,
            samples,
            end_delay,
            stop_sender,
            task,
        } = self;

        let _ = stop_sender.send(true);
        let _ = task.await;

        let end_memory = api.admin_status().await?.1.memory;
        {
            if let Ok(mut guard) = samples.lock() {
                guard.push(end_memory);
            }
        }

        tokio::time::sleep(end_delay).await;

        let end_plus_delay_memory = api.admin_status().await?.1.memory;
        {
            if let Ok(mut guard) = samples.lock() {
                guard.push(end_plus_delay_memory);
            }
        }

        let samples = samples
            .lock()
            .map_err(|_| BenchError {
                description: "failed to collect memory samples".to_string(),
            })?
            .clone();

        if samples.is_empty() {
            return Err(BenchError {
                description: "no memory samples collected".to_string(),
            });
        }

        let peak = samples.iter().max().copied().unwrap_or_default();

        Ok(MemoryStats {
            start: samples[0],
            peak,
            end: end_memory,
            end_plus_delay: end_plus_delay_memory,
        })
    }
}
