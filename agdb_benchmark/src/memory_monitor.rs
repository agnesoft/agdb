use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::database::api_with_client;
use crate::results::MemoryStats;
use crate::retry::with_retry;
use agdb_api::AgdbApi;
use agdb_api::ReqwestClient;
use reqwest::Client;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub(crate) struct ServerMemoryMonitor {
    api: AgdbApi<ReqwestClient>,
    start_memory: u64,
    peak_memory: Arc<AtomicU64>,
    retry: crate::config::RetryConfig,
    end_delay: Duration,
    stop_sender: watch::Sender<bool>,
    task: JoinHandle<()>,
}

impl ServerMemoryMonitor {
    pub(crate) async fn start(
        config: &Config,
        address: &str,
        admin_username: &str,
        admin_password: &str,
        client: &Client,
    ) -> BenchResult<Self> {
        let mut api = api_with_client(client.clone(), address);
        api.user_login(admin_username, admin_password).await?;

        let start_memory = api.admin_status().await?.1.memory;
        let peak_memory = Arc::new(AtomicU64::new(start_memory));
        let interval = Duration::from_millis(config.server.memory_poll_interval_ms.max(1));
        let end_delay = Duration::from_millis(config.server.memory_end_delay_ms.max(1));
        let retry = config.server.retry.max_consecutive_failures;
        let (stop_sender, mut stop_receiver) = watch::channel(false);

        let address = address.to_string();
        let client = client.clone();

        let peak = peak_memory.clone();
        let username = admin_username.to_string();
        let password = admin_password.to_string();

        let task = tokio::spawn(async move {
            let mut failures = 0u32;
            let mut task_api = api_with_client(client, &address);
            if task_api.user_login(&username, &password).await.is_err() {
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
                                if status.memory > peak.load(Ordering::Relaxed) {
                                    peak.store(status.memory, Ordering::Relaxed);
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
            start_memory,
            peak_memory,
            retry: config.server.retry,
            end_delay,
            stop_sender,
            task,
        })
    }

    pub(crate) async fn finish(self) -> BenchResult<MemoryStats> {
        let _ = self.stop_sender.send(true);
        let _ = self.task.await;

        let mut api = self.api;

        let end = with_retry(&self.retry, "memory monitor end sample", &mut api, |api| {
            Box::pin(async { Ok(api.admin_status().await?.1.memory) })
        })
        .await
        .unwrap_or(0);

        tokio::time::sleep(self.end_delay).await;

        let end_plus_delay = with_retry(
            &self.retry,
            "memory monitor delayed sample",
            &mut api,
            |api| Box::pin(async { Ok(api.admin_status().await?.1.memory) }),
        )
        .await
        .unwrap_or(end);

        Ok(MemoryStats {
            start: self.start_memory,
            peak: self.peak_memory.load(Ordering::Relaxed),
            end,
            end_plus_delay,
        })
    }
}
