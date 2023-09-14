use crate::bench_result::BenchResult;
use crate::config::Config;
use num_format::Locale;
use num_format::ToFormattedString;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

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
    } else if (KB) < bytes {
        format!("{} kB", (bytes / KB).to_formatted_string(&locale))
    } else {
        format!("{} b", bytes.to_formatted_string(&locale))
    }
}

pub(crate) fn measured(mut predicate: impl FnMut() -> BenchResult<()>) -> BenchResult<Duration> {
    let start = Instant::now();
    predicate()?;
    let duration = start.elapsed();
    Ok(duration)
}

pub(crate) fn print_flush(message: String) {
    print!("{message}");
    std::io::stdout().flush().unwrap();
}

pub(crate) fn print_header(config: &Config) {
    let padding = config.padding as usize;
    let cell_padding = config.cell_padding as usize;

    println!(
        "{:padding$} | {:^cell_padding$} | {:^cell_padding$} | {:^cell_padding$} | {:^cell_padding$} | {:^cell_padding$} | {:^cell_padding$} | {:^cell_padding$} | {:^cell_padding$}",
        "Description", "Threads", "Iters", "Per iter", "Count", "Min", "Avg", "Max", "Total"
    );
    println!(
        "{:-<padding$} | {:-<cell_padding$} | {:-<cell_padding$} | {:-<cell_padding$} | {:-<cell_padding$} | {:-<cell_padding$} | {:-<cell_padding$} | {:-<cell_padding$} | {:-<cell_padding$}",
        "", "", "", "", "", "", "", "", ""
    );
}

pub(crate) fn report(
    description: &str,
    threads: u64,
    per_thread: u64,
    per_action: u64,
    mut times: Vec<Duration>,
    total: Duration,
    config: &Config,
) {
    let zero_time: Duration = Duration::default();

    times.sort();

    let min = times.first().unwrap_or(&zero_time);
    let max = times.last().unwrap_or(&zero_time);
    let avg = if times.is_empty() {
        zero_time
    } else {
        times.iter().sum::<Duration>() / times.len() as u32
    };

    let padding = config.padding as usize;
    let cell_padding = config.cell_padding as usize;

    println!("{:padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$} | {:cell_padding$}",
        description, threads.to_formatted_string(&config.locale), per_thread.to_formatted_string(&config.locale), per_action.to_formatted_string(&config.locale), times.len().to_formatted_string(&config.locale), format_duration(*min, config.locale), format_duration(avg, config.locale), format_duration(*max, config.locale), format_duration(total, config.locale)
        )
}
