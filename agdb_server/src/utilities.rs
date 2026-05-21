use crate::server_error::ServerResult;
use agdb::QueryType;
use agdb_api::DbUserRole;
use agdb_api::Queries;
use std::cell::RefCell;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

thread_local! {
    static TIMESTAMP_CACHE: RefCell<(u64, String)> = const { RefCell::new((u64::MAX, String::new())) };
}

pub(crate) async fn get_size<P>(path: P) -> ServerResult<u64>
where
    P: AsRef<Path>,
{
    let path_metadata = path.as_ref().symlink_metadata()?;
    let mut size_in_bytes = 0;

    if path_metadata.is_dir() {
        let mut read_dir = tokio::fs::read_dir(&path).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let entry_metadata = entry.metadata().await?;

            if entry_metadata.is_dir() {
                size_in_bytes += Box::pin(get_size(entry.path())).await?;
            } else {
                size_in_bytes += entry_metadata.len();
            }
        }
    } else {
        size_in_bytes = path_metadata.len();
    }

    Ok(size_in_bytes)
}

pub(crate) fn remove_file_if_exists<P>(file: P) -> ServerResult
where
    P: AsRef<Path>,
{
    if std::fs::exists(&file)? {
        std::fs::remove_file(file)?;
    }

    Ok(())
}

pub(crate) fn required_role(queries: &Queries) -> DbUserRole {
    for q in &queries.0 {
        match q {
            QueryType::InsertAlias(_)
            | QueryType::InsertEdges(_)
            | QueryType::InsertNodes(_)
            | QueryType::InsertValues(_)
            | QueryType::Remove(_)
            | QueryType::RemoveAliases(_)
            | QueryType::RemoveValues(_) => {
                return DbUserRole::Write;
            }
            _ => {}
        }
    }

    DbUserRole::Read
}

pub(crate) fn unquote(value: &str) -> &str {
    value.trim_start_matches('"').trim_end_matches('"')
}

pub(crate) fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    TIMESTAMP_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if cache.0 != secs {
            cache.0 = secs;
            cache.1 = format_unix_timestamp(secs);
        }

        cache.1.clone()
    })
}

fn format_unix_timestamp(secs: u64) -> String {
    let (year, month, day, hour, min, sec) = secs_to_datetime(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn secs_to_datetime(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = secs % 60;
    let min = (secs / 60) % 60;
    let hour = (secs / 3600) % 24;
    let days = secs / 86_400;
    let (year, month, day) = civil_from_unix_days(days);

    (year, month, day, hour, min, sec)
}

// Convert days since Unix epoch (1970-01-01) to a civil date in constant time.
// Based on Howard Hinnant's civil calendar conversion algorithm.
fn civil_from_unix_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]

    if month <= 2 {
        year += 1;
    }

    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_size_on_file() -> anyhow::Result<()> {
        let size = get_size("Cargo.toml").await.unwrap();
        assert_ne!(size, 0);
        Ok(())
    }

    #[test]
    fn timestamp_format() {
        let ts = timestamp();
        assert!(ts.ends_with('Z'));
        assert_eq!(ts.len(), 20);
    }

    #[test]
    fn secs_to_datetime_epoch() {
        assert_eq!(secs_to_datetime(0), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn secs_to_datetime_known_date() {
        assert_eq!(secs_to_datetime(1704067200), (2024, 1, 1, 0, 0, 0));
    }

    #[test]
    fn secs_to_datetime_leap_day() {
        assert_eq!(secs_to_datetime(1709164800), (2024, 2, 29, 0, 0, 0));
    }

    #[test]
    fn format_unix_timestamp_known_date() {
        assert_eq!(format_unix_timestamp(1704067200), "2024-01-01T00:00:00Z");
    }
}
