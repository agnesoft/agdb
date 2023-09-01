use crate::bench_result::BenchResult;
use crate::CELL_PADDING;
use crate::LOCALE;
use crate::PADDING;
use num_format::ToFormattedString;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

pub(crate) fn format_duration(duration: Duration) -> String {
    if duration.as_micros() < 1000 {
        format!(
            "{} μs (0 ms)",
            duration.as_micros().to_formatted_string(&LOCALE)
        )
    } else if duration.as_millis() < 1000 {
        format!("{} ms", duration.as_millis().to_formatted_string(&LOCALE))
    } else {
        format!("{} s", duration.as_secs().to_formatted_string(&LOCALE))
    }
}

pub(crate) fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if (10 * MB) < bytes {
        format!("{} MB", (bytes / MB).to_formatted_string(&LOCALE))
    } else if (KB) < bytes {
        format!("{} kB", (bytes / KB).to_formatted_string(&LOCALE))
    } else {
        format!("{} b", bytes.to_formatted_string(&LOCALE))
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

pub(crate) fn print_header() {
    println!(
        "{:PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$}",
        "Description", "Count", "Min", "Avg", "Max", "Total"
    );
    println!(
        "{:-<PADDING$} | {:-<CELL_PADDING$} | {:-<CELL_PADDING$} | {:-<CELL_PADDING$} | {:-<CELL_PADDING$} | {:-<CELL_PADDING$}",
        "", "", "", "", "", ""
    );
}

pub(crate) fn report(description: &str, mut times: Vec<Duration>) {
    let zero_time: Duration = Duration::default();

    times.sort();

    let min = times.first().unwrap_or(&zero_time);
    let max = times.last().unwrap_or(&zero_time);
    let total = times.iter().sum::<Duration>();
    let avg = if times.is_empty() {
        zero_time
    } else {
        total / times.len() as u32
    };

    println!("{:PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$} | {:CELL_PADDING$}",
        description, times.len().to_formatted_string(&LOCALE), format_duration(*min), format_duration(avg), format_duration(*max), format_duration(total)
        )
}
