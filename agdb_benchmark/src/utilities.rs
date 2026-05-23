use crate::bench_result::BenchResult;
use crate::config::Config;
use crate::results::TargetKind;
use crate::results::TargetResult;
use num_format::Locale;
use num_format::ToFormattedString;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub(crate) fn format_duration(duration: Duration, locale: Locale) -> String {
    if duration.as_micros() < 1000 {
        format!("{} μs", duration.as_micros().to_formatted_string(&locale))
    } else if duration.as_millis() < 1000 {
        format!("{} ms", duration.as_millis().to_formatted_string(&locale))
    } else {
        format!("{} s", duration.as_secs().to_formatted_string(&locale))
    }
}

pub(crate) fn format_size(bytes: u64, locale: Locale) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if (10 * MB) < bytes {
        format!("{} MB", (bytes / MB).to_formatted_string(&locale))
    } else if KB < bytes {
        format!("{} kB", (bytes / KB).to_formatted_string(&locale))
    } else {
        format!("{} b", bytes.to_formatted_string(&locale))
    }
}

pub(crate) fn measured(mut predicate: impl FnMut() -> BenchResult<()>) -> BenchResult<Duration> {
    let start = Instant::now();
    predicate()?;
    Ok(start.elapsed())
}

pub(crate) async fn measured_async<F>(future: F) -> BenchResult<Duration>
where
    F: std::future::Future<Output = BenchResult<()>>,
{
    let start = Instant::now();
    future.await?;
    Ok(start.elapsed())
}

pub(crate) fn print_flush(message: String) {
    print!("{message}");
    std::io::stdout().flush().unwrap();
}

pub(crate) struct ProgressIndicator {
    stop_sender: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

impl ProgressIndicator {
    pub(crate) fn start(label: &str) -> Self {
        print!("{label}: ");
        std::io::stdout().flush().unwrap();

        let (stop_sender, mut stop_receiver) = oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut stop_receiver => {
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(10)) => {
                        print!(".");
                        std::io::stdout().flush().unwrap();
                    }
                }
            }
        });

        Self {
            stop_sender: Some(stop_sender),
            task,
        }
    }

    pub(crate) async fn finish(mut self, status: &str) {
        if let Some(stop_sender) = self.stop_sender.take() {
            let _ = stop_sender.send(());
        }

        let _ = self.task.await;
        println!(" {status}");
    }
}

pub(crate) fn print_final_summary(config: &Config, results: &[TargetResult]) {
    let embedded = find_result(results, &TargetKind::Embedded);
    let local = find_result(results, &TargetKind::LocalServer);
    let remote = find_result(results, &TargetKind::RemoteServer);

    let rows = vec![
        (
            "Status".to_string(),
            status_cell(embedded),
            status_cell(local),
            status_cell(remote),
        ),
        (
            "Address".to_string(),
            address_cell(embedded),
            address_cell(local),
            address_cell(remote),
        ),
        (
            "Time".to_string(),
            time_cell(embedded, config.locale),
            time_cell(local, config.locale),
            time_cell(remote, config.locale),
        ),
        (
            "Requests (write/read)".to_string(),
            requests_cell(embedded, config),
            requests_cell(local, config),
            requests_cell(remote, config),
        ),
        (
            "Post Writers (min/avg/max)".to_string(),
            workload_cell(embedded.map(|r| &r.workload.post_writers), config.locale),
            workload_cell(local.map(|r| &r.workload.post_writers), config.locale),
            workload_cell(remote.map(|r| &r.workload.post_writers), config.locale),
        ),
        (
            "Comment Writers (min/avg/max)".to_string(),
            workload_cell(embedded.map(|r| &r.workload.comment_writers), config.locale),
            workload_cell(local.map(|r| &r.workload.comment_writers), config.locale),
            workload_cell(remote.map(|r| &r.workload.comment_writers), config.locale),
        ),
        (
            "Post Readers (min/avg/max)".to_string(),
            workload_cell(embedded.map(|r| &r.workload.post_readers), config.locale),
            workload_cell(local.map(|r| &r.workload.post_readers), config.locale),
            workload_cell(remote.map(|r| &r.workload.post_readers), config.locale),
        ),
        (
            "Comment Readers (min/avg/max)".to_string(),
            workload_cell(embedded.map(|r| &r.workload.comment_readers), config.locale),
            workload_cell(local.map(|r| &r.workload.comment_readers), config.locale),
            workload_cell(remote.map(|r| &r.workload.comment_readers), config.locale),
        ),
        (
            "Db (size/optimized)".to_string(),
            db_cell(embedded, config.locale),
            db_cell(local, config.locale),
            db_cell(remote, config.locale),
        ),
        (
            format!(
                "Mem (start/peak/end/+{}s)",
                config.server.memory_end_delay_ms / 1000
            ),
            mem_cell(embedded, config.locale),
            mem_cell(local, config.locale),
            mem_cell(remote, config.locale),
        ),
        (
            "Error".to_string(),
            error_cell(embedded),
            error_cell(local),
            error_cell(remote),
        ),
    ];

    let metric_width = rows
        .iter()
        .map(|(metric, _, _, _)| metric.len())
        .max()
        .unwrap_or(10)
        .max("Metric".len());
    let embedded_width = rows
        .iter()
        .map(|(_, embedded, _, _)| embedded.len())
        .max()
        .unwrap_or(8)
        .max("Embedded".len());
    let local_width = rows
        .iter()
        .map(|(_, _, local, _)| local.len())
        .max()
        .unwrap_or(12)
        .max("Local Server".len());
    let remote_width = rows
        .iter()
        .map(|(_, _, _, remote)| remote.len())
        .max()
        .unwrap_or(13)
        .max("Remote Server".len());

    println!(
        "{:<metric_width$} | {:<embedded_width$} | {:<local_width$} | {:<remote_width$}",
        "Metric", "Embedded", "Local Server", "Remote Server"
    );
    println!(
        "{:-<metric_width$}-+-{:-<embedded_width$}-+-{:-<local_width$}-+-{:-<remote_width$}",
        "", "", "", ""
    );

    for (metric, embedded, local, remote) in rows {
        println!(
            "{:<metric_width$} | {:<embedded_width$} | {:<local_width$} | {:<remote_width$}",
            metric, embedded, local, remote
        );
    }
}

fn find_result<'a>(results: &'a [TargetResult], kind: &TargetKind) -> Option<&'a TargetResult> {
    results.iter().find(|result| {
        matches!(
            (&result.kind, kind),
            (TargetKind::Embedded, TargetKind::Embedded)
                | (TargetKind::LocalServer, TargetKind::LocalServer)
                | (TargetKind::RemoteServer, TargetKind::RemoteServer)
        )
    })
}

fn status_cell(result: Option<&TargetResult>) -> String {
    match result {
        None => "-".to_string(),
        Some(result) if result.error.is_some() => "failed".to_string(),
        Some(_) => "done".to_string(),
    }
}

fn address_cell(result: Option<&TargetResult>) -> String {
    result
        .and_then(|r| r.address.clone())
        .unwrap_or_else(|| "-".to_string())
}

fn time_cell(result: Option<&TargetResult>, locale: Locale) -> String {
    result
        .map(|r| format_duration(r.total, locale))
        .unwrap_or_else(|| "-".to_string())
}

fn workload_cell(timing: Option<&crate::results::TimingStats>, locale: Locale) -> String {
    if let Some(timing) = timing
        && timing.count != 0
    {
        return format!(
            "{} / {} / {}",
            format_duration(timing.min, locale),
            format_duration(timing.average(), locale),
            format_duration(timing.max, locale)
        );
    }

    "-".to_string()
}

fn requests_cell(result: Option<&TargetResult>, config: &Config) -> String {
    if result.is_none() {
        return "-".to_string();
    }

    let writes = config
        .posters
        .count
        .saturating_mul(config.posters.posts)
        .saturating_add(
            config
                .commenters
                .count
                .saturating_mul(config.commenters.comments),
        );
    let reads = config
        .post_readers
        .count
        .saturating_mul(config.post_readers.reads_per_reader)
        .saturating_add(
            config
                .comment_readers
                .count
                .saturating_mul(config.comment_readers.reads_per_reader),
        );

    format!(
        "{} / {}",
        writes.to_formatted_string(&config.locale),
        reads.to_formatted_string(&config.locale)
    )
}

fn db_cell(result: Option<&TargetResult>, locale: Locale) -> String {
    if let Some(result) = result
        && let (Some(size), Some(optimized)) = (result.database_before, result.database_after)
    {
        return format!(
            "{} / {}",
            format_size(size, locale),
            format_size(optimized, locale)
        );
    }

    "-".to_string()
}

fn mem_cell(result: Option<&TargetResult>, locale: Locale) -> String {
    if let Some(result) = result
        && let Some(memory) = &result.memory
    {
        return format!(
            "{} / {} / {} / {}",
            format_size(memory.start, locale),
            format_size(memory.peak, locale),
            format_size(memory.end, locale),
            format_size(memory.end_plus_delay, locale)
        );
    }

    "-".to_string()
}

fn error_cell(result: Option<&TargetResult>) -> String {
    result
        .and_then(|r| r.error.clone())
        .unwrap_or_else(|| "-".to_string())
}
