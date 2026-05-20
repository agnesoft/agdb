use crate::server_error::ServerResult;
use agdb::QueryType;
use agdb_api::DbUserRole;
use agdb_api::Queries;
use std::path::Path;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

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
    format_system_time(&SystemTime::now())
}

pub(crate) fn format_system_time(time: &SystemTime) -> String {
    let secs = time
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (year, month, day, hour, min, sec) = secs_to_datetime(secs);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

fn secs_to_datetime(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let sec = secs % 60;
    let min = (secs / 60) % 60;
    let hour = (secs / 3600) % 24;

    let mut days = secs / 86400;
    let mut year = 1970u64;

    loop {
        let days_in_year = if is_leap(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let leap = is_leap(year);
    let month_days: [u64; 12] = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 0u64;
    for md in &month_days {
        if days < *md {
            break;
        }
        days -= *md;
        month += 1;
    }

    (year, month + 1, days + 1, hour, min, sec)
}

fn is_leap(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
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
}
