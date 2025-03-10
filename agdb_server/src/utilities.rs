use crate::server_error::ServerResult;
use agdb::QueryType;
use agdb_api::DbUserRole;
use agdb_api::Queries;
use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_size_on_file() -> anyhow::Result<()> {
        let size = get_size("Cargo.toml").await.unwrap();
        assert_ne!(size, 0);
        Ok(())
    }
}
