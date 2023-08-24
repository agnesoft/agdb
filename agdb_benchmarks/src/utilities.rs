use crate::bench_result::BenchResult;
use crate::LOCALE;
use num_format::ToFormattedString;
use std::io::Write;
use std::time::Duration;
use std::time::Instant;

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

const KB: u64 = 1024;
const MB: u64 = KB * 1024;

pub(crate) fn format_size(bytes: u64) -> String {
    if (10 * MB) < bytes {
        format!("{} MB", (bytes / MB).to_formatted_string(&LOCALE))
    } else if (KB) < bytes {
        format!("{} kB", (bytes / KB).to_formatted_string(&LOCALE))
    } else {
        format!("{} b", bytes.to_formatted_string(&LOCALE))
    }
}
