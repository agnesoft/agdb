use crate::action::ClusterAction;
use crate::cluster::Cluster;
use crate::config::Config;
use crate::raft::Log;
use crate::raft::Storage;
use crate::server_db::SERVER_DB_FILE;
use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use agdb_api::DbKind;
use axum::http::StatusCode;
use crc::CRC_64_XZ;
use crc::Crc;
use futures::StreamExt;
use std::path::Component;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

pub(crate) const RESYNC_SERVER_DB_OWNER: &str = "";
pub(crate) const RESYNC_SERVER_DB_NAME: &str = "server_db";

const SNAPSHOT_PARTIAL_TTL_SECS: u64 = 3600;

// ---------------------------------------------------------------------------
// Full-node resync infrastructure
// ---------------------------------------------------------------------------

pub(crate) async fn resync_from_leader(
    cluster: &Cluster,
    config: &Config,
    force_snapshot: bool,
) -> ServerResult<()> {
    let leader = cluster.raft.read().await.leader();
    let leader_index = match leader {
        Some(l) if l as usize != cluster.index => l as usize,
        _ => {
            return Err(ServerError::from(
                "Cannot resync: no leader available or this node is the leader",
            ));
        }
    };

    cluster.resync.store(true, Ordering::Relaxed);

    if !force_snapshot {
        let from_index = cluster.raft.read().await.storage.log_commit();
        match catchup_logs_from_leader(cluster, config, leader_index, from_index).await {
            Ok(()) => {
                cluster.resync.store(false, Ordering::Relaxed);
                crate::info!("[{}] Resync completed via log catch-up", cluster.index);
                return Ok(());
            }
            Err(e) => {
                crate::info!(
                    "[{}] Log catch-up failed ({e:?}), falling back to full snapshot",
                    cluster.index
                );
            }
        }
    }

    let mut snapshot_sources: Vec<usize> = (0..cluster.nodes.len())
        .filter(|index| *index != cluster.index && *index != leader_index)
        .collect();
    snapshot_sources.push(leader_index);

    crate::info!(
        "[{}] Starting snapshot resync, candidates: {:?}",
        cluster.index,
        snapshot_sources
    );

    let mut result = Err(ServerError::from("no snapshot source available"));

    for source_index in snapshot_sources {
        crate::info!(
            "[{}] Attempting snapshot download from node {}",
            cluster.index,
            source_index
        );

        match do_resync(cluster, config, source_index).await {
            Ok(()) => {
                result = Ok(());
                break;
            }
            Err(error) => {
                crate::warn!(
                    "[{}] Snapshot download from node {} failed: {:?}",
                    cluster.index,
                    source_index,
                    error
                );
                result = Err(error);
            }
        }
    }

    cluster.resync.store(false, Ordering::Relaxed);

    match &result {
        Ok(()) => crate::info!("[{}] Resync completed via snapshot", cluster.index),
        Err(e) => crate::error!("[{}] Resync failed: {:?}", cluster.index, e),
    }

    result
}

async fn catchup_logs_from_leader(
    cluster: &Cluster,
    config: &Config,
    leader_index: usize,
    from_index: u64,
) -> ServerResult<()> {
    let logs_url = format!(
        "{}/api/v1/cluster/logs?from_index={from_index}",
        cluster.nodes[leader_index].base_url()
    );

    let client = cluster.nodes[leader_index].http_client();
    let response = client
        .get(&logs_url)
        .bearer_auth(&config.cluster_token)
        .timeout(Duration::from_secs(600))
        .send()
        .await
        .map_err(|e| ServerError::from(format!("log catch-up request failed: {e:?}")))?;

    let status = response.status().as_u16();
    if status != 200 {
        let body = response.text().await.unwrap_or_default();
        return Err(ServerError::from(format!(
            "log catch-up endpoint returned {status}: {body}",
        )));
    }

    let mut stream = response.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();

    while buf.len() < 16 {
        match stream.next().await {
            Some(Ok(chunk)) => buf.extend_from_slice(&chunk),
            Some(Err(e)) => {
                return Err(ServerError::from(format!(
                    "log catch-up stream error: {e:?}"
                )));
            }
            None => {
                return Err(ServerError::from(
                    "log catch-up response too small (missing header)",
                ));
            }
        }
    }

    let entry_count = u64::from_le_bytes(buf[0..8].try_into().unwrap());
    let commit_index = u64::from_le_bytes(buf[8..16].try_into().unwrap());

    const MAX_CATCHUP_ENTRIES: u64 = 100_000;
    if entry_count > MAX_CATCHUP_ENTRIES {
        return Err(ServerError::from(format!(
            "log catch-up entry count {entry_count} exceeds limit {MAX_CATCHUP_ENTRIES}",
        )));
    }

    buf.drain(0..16);

    let mut entries = Vec::with_capacity(entry_count as usize);
    let mut prev_index = from_index;

    for _ in 0..entry_count {
        while buf.len() < 8 {
            match stream.next().await {
                Some(Ok(chunk)) => buf.extend_from_slice(&chunk),
                Some(Err(e)) => {
                    return Err(ServerError::from(format!(
                        "log catch-up stream error: {e:?}"
                    )));
                }
                None => return Err(ServerError::from("log catch-up truncated (json_len)")),
            }
        }
        let json_len = u64::from_le_bytes(buf[0..8].try_into().unwrap()) as usize;
        buf.drain(0..8);

        while buf.len() < json_len {
            match stream.next().await {
                Some(Ok(chunk)) => buf.extend_from_slice(&chunk),
                Some(Err(e)) => {
                    return Err(ServerError::from(format!(
                        "log catch-up stream error: {e:?}"
                    )));
                }
                None => return Err(ServerError::from("log catch-up truncated (json_bytes)")),
            }
        }
        let log: Log<ClusterAction> = serde_json::from_slice(&buf[..json_len])
            .map_err(|e| ServerError::from(format!("log catch-up deserialization error: {e}")))?;
        buf.drain(0..json_len);

        if log.index != prev_index + 1 {
            return Err(ServerError::from(format!(
                "log catch-up non-contiguous: expected index {}, got {}",
                prev_index + 1,
                log.index
            )));
        }

        prev_index = log.index;
        entries.push(log);
    }

    let mut raft = cluster.raft.write().await;
    let last_index = entries.last().map(|log| log.index);

    for log in entries {
        raft.storage.append(log, None).await?;
    }

    if let Some(last_index) = last_index {
        if commit_index > last_index {
            return Err(ServerError::from(format!(
                "log catch-up commit_index {commit_index} exceeds last applied index {last_index}",
            )));
        }

        if commit_index > raft.storage.commit {
            raft.storage.commit(commit_index).await?;
        }
    }

    raft.refresh_local_from_storage();

    crate::info!(
        "[{}] Log catch-up complete: {} entries applied, commit_index={}",
        cluster.index,
        entry_count,
        commit_index
    );

    Ok(())
}

async fn validate_snapshot_header(
    partial_file: &std::path::Path,
    current_commit: u64,
) -> ServerResult<()> {
    let mut file = tokio::fs::File::open(partial_file).await?;
    let mut header = [0u8; 32];
    file.read_exact(&mut header)
        .await
        .map_err(|_| ServerError::from("snapshot header missing or truncated"))?;
    let header_commit = u64::from_le_bytes(header[16..24].try_into().unwrap());
    if header_commit < current_commit {
        return Err(ServerError::from(format!(
            "snapshot commit {header_commit} is behind current commit {current_commit}"
        )));
    }
    Ok(())
}

async fn do_resync(cluster: &Cluster, config: &Config, node_index: usize) -> ServerResult<()> {
    let data_dir = std::path::Path::new(&config.data_dir);
    let dir_name = data_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let partial_dir = data_dir.with_file_name(format!("{dir_name}.snapshot_partial"));
    let install_dir = data_dir.with_file_name(format!("{dir_name}.snapshot_install"));
    let backup_dir = data_dir.with_file_name(format!("{dir_name}.snapshot_bak"));

    if install_dir.exists() {
        let _ = std::fs::remove_dir_all(&install_dir);
    }

    cleanup_stale_partial(&partial_dir);
    download_snapshot_to_partial(cluster, config, node_index, &partial_dir).await?;

    let partial_file = partial_dir.join("data.bin");
    let current_commit = cluster.raft.read().await.storage.commit;
    validate_snapshot_header(&partial_file, current_commit).await?;

    if let Err(e) = extract_snapshot_binary(
        &partial_file,
        &install_dir,
        config.cluster_max_chunk_size as usize,
    )
    .await
    {
        let _ = std::fs::remove_dir_all(&install_dir);
        return Err(e);
    }

    if backup_dir.exists() {
        let _ = std::fs::remove_dir_all(&backup_dir);
    }

    let mut raft = cluster.raft.write().await;
    raft.storage.close_handles(&partial_dir).await?;

    if data_dir.exists() {
        std::fs::rename(data_dir, &backup_dir)?;
    }

    if let Err(e) = std::fs::rename(&install_dir, data_dir) {
        let _ = std::fs::rename(&backup_dir, data_dir);
        return Err(ServerError::from(format!(
            "failed to install snapshot: {e}"
        )));
    }

    if let Err(e) = raft.storage.reinit(config).await {
        let _ = std::fs::remove_dir_all(data_dir);
        let _ = std::fs::rename(&backup_dir, data_dir);
        let _ = std::fs::remove_dir_all(&partial_dir);
        drop(raft);
        return Err(e);
    }

    raft.refresh_local_from_storage();
    drop(raft);

    let _ = std::fs::remove_dir_all(&backup_dir);
    let _ = std::fs::remove_dir_all(&partial_dir);

    Ok(())
}

async fn download_snapshot_to_partial(
    cluster: &Cluster,
    config: &Config,
    node_index: usize,
    partial_dir: &std::path::Path,
) -> ServerResult<()> {
    std::fs::create_dir_all(partial_dir)?;
    let partial_file = partial_dir.join("data.bin");
    let id_file = partial_dir.join(".id");

    let (resume_offset, resume_id) = if partial_file.exists() && id_file.exists() {
        let offset = partial_file.metadata()?.len();
        let id = std::fs::read_to_string(&id_file).unwrap_or_default();
        (offset, id.trim().to_string())
    } else {
        (0u64, String::new())
    };

    let snapshot_url = format!(
        "{}/api/v1/cluster/snapshot",
        cluster.nodes[node_index].base_url()
    );

    let client = cluster.nodes[node_index].http_client();
    let mut request = client
        .get(&snapshot_url)
        .bearer_auth(&config.cluster_token)
        .timeout(Duration::from_secs(600));

    if resume_offset > 0 && !resume_id.is_empty() {
        request = request
            .header("range", format!("bytes={resume_offset}-"))
            .header("x-snapshot-resume-id", &resume_id);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ServerError::from(format!("snapshot request failed: {e:?}")))?;

    let http_status = response.status().as_u16();
    if http_status != 200 && http_status != 206 {
        return Err(ServerError::from(format!(
            "snapshot endpoint returned {http_status}"
        )));
    }

    let server_id = response
        .headers()
        .get("x-snapshot-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let is_resume = http_status == 206 && !server_id.is_empty() && server_id == resume_id;

    if !is_resume {
        let _ = std::fs::remove_file(&partial_file);
    }

    std::fs::write(&id_file, &server_id)?;

    let append = is_resume && partial_file.exists();
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .open(&partial_file)
        .await?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ServerError::from(format!("stream read error: {e:?}")))?;
        file.write_all(&chunk).await?;
    }
    file.flush().await?;

    Ok(())
}

pub(crate) async fn extract_snapshot_binary(
    partial_file: &std::path::Path,
    install_dir: &std::path::Path,
    chunk_size: usize,
) -> ServerResult<()> {
    std::fs::create_dir_all(install_dir)?;

    let mut file = tokio::fs::File::open(partial_file).await?;

    let crc64 = Crc::<u64>::new(&CRC_64_XZ);
    let mut digest = crc64.digest();

    let mut header = [0u8; 32];
    file.read_exact(&mut header)
        .await
        .map_err(|_| ServerError::from("snapshot too small (missing header)"))?;
    digest.update(&header);

    let file_count = u64::from_le_bytes(header[24..32].try_into().unwrap());

    const MAX_SNAPSHOT_FILES: u64 = 100_000;

    if file_count > MAX_SNAPSHOT_FILES {
        return Err(ServerError::from(format!(
            "snapshot file_count {file_count} exceeds sanity limit {MAX_SNAPSHOT_FILES}"
        )));
    }

    let canonical_install = install_dir.canonicalize()?;
    let mut buf = vec![0u8; chunk_size];

    for _ in 0..file_count {
        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)
            .await
            .map_err(|_| ServerError::from("snapshot truncated (path_len)"))?;
        digest.update(&len_buf);
        let path_len = u32::from_le_bytes(len_buf) as usize;

        if path_len == 0 || path_len > 4096 {
            return Err(ServerError::from(format!(
                "invalid path length in snapshot: {path_len}"
            )));
        }

        let mut path_buf = vec![0u8; path_len];
        file.read_exact(&mut path_buf)
            .await
            .map_err(|_| ServerError::from("snapshot truncated (path)"))?;
        digest.update(&path_buf);
        let rel_path = String::from_utf8(path_buf)
            .map_err(|e| ServerError::from(format!("invalid path encoding: {e}")))?;

        let mut file_len_buf = [0u8; 8];
        file.read_exact(&mut file_len_buf)
            .await
            .map_err(|_| ServerError::from("snapshot truncated (file_len)"))?;
        digest.update(&file_len_buf);
        let file_len = u64::from_le_bytes(file_len_buf);

        let abs = canonical_install.join(&rel_path);
        if abs.components().any(|c| c == Component::ParentDir)
            || !abs.starts_with(&canonical_install)
        {
            return Err(ServerError::from(format!(
                "invalid path in snapshot: {rel_path}"
            )));
        }

        if let Some(parent) = abs.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut out = tokio::fs::File::create(&abs).await?;
        let mut remaining = file_len;

        while remaining > 0 {
            let to_read = std::cmp::min(remaining, buf.len() as u64) as usize;
            file.read_exact(&mut buf[..to_read])
                .await
                .map_err(|_| ServerError::from("snapshot truncated (file_data)"))?;
            digest.update(&buf[..to_read]);
            out.write_all(&buf[..to_read]).await?;
            remaining -= to_read as u64;
        }
        out.sync_data().await?;
    }

    // CRC64 trailer: 8 bytes at end (optional, backward compatible)
    let computed_crc = digest.finalize();
    let mut crc_buf = [0u8; 8];
    // No CRC trailer → old format or resumed stream; skip verification
    if file.read_exact(&mut crc_buf).await.is_ok() {
        let expected_crc = u64::from_le_bytes(crc_buf);
        if computed_crc != expected_crc {
            return Err(ServerError::from(format!(
                "snapshot CRC64 mismatch: computed {computed_crc:#018x}, expected {expected_crc:#018x}"
            )));
        }
    }

    Ok(())
}

fn cleanup_stale_partial(partial_dir: &std::path::Path) {
    if !partial_dir.exists() {
        return;
    }
    let Ok(metadata) = partial_dir.metadata() else {
        return;
    };
    let Ok(modified) = metadata.modified() else {
        return;
    };
    let Ok(age) = std::time::SystemTime::now().duration_since(modified) else {
        return;
    };
    if age.as_secs() >= SNAPSHOT_PARTIAL_TTL_SECS {
        let _ = std::fs::remove_dir_all(partial_dir);
    }
}

// ---------------------------------------------------------------------------
// Per-DB resync infrastructure
// ---------------------------------------------------------------------------

struct TempDirGuard(PathBuf);

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn validate_resync_name(name: &str, label: &str) -> ServerResult<()> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
        || name == "."
        || name == ".."
        || name.contains("..")
    {
        return Err(ServerError::from(format!(
            "per-DB resync: invalid {label}: {name:?}"
        )));
    }
    Ok(())
}

pub(crate) async fn resync_single_db(
    cluster: &Cluster,
    config: &Config,
    owner: &str,
    db: &str,
) -> ServerResult<()> {
    let is_server_db = owner == RESYNC_SERVER_DB_OWNER && db == RESYNC_SERVER_DB_NAME;

    if !is_server_db {
        validate_resync_name(owner, "owner")?;
        validate_resync_name(db, "db")?;
    }

    let (leader_index, snapshot_lock, db_pool, server_db) = {
        let raft = cluster.raft.read().await;
        let leader = raft
            .leader()
            .ok_or_else(|| ServerError::from("per-DB resync: no leader available"))?;

        if leader as usize == cluster.index {
            return Err(ServerError::from("per-DB resync: this node is the leader"));
        }

        (
            leader as usize,
            raft.storage.snapshot_lock.clone(),
            raft.storage.db_pool.clone(),
            raft.storage.db.clone(),
        )
    };

    let leader_base_url = cluster.nodes[leader_index].base_url();

    let url = if is_server_db {
        format!("{leader_base_url}/api/v1/cluster/snapshot/server_db")
    } else {
        format!("{leader_base_url}/api/v1/cluster/snapshot/{owner}/{db}")
    };

    let client = cluster.nodes[leader_index].http_client();
    let response = client
        .get(&url)
        .bearer_auth(&config.cluster_token)
        .timeout(Duration::from_secs(600))
        .send()
        .await
        .map_err(|e| ServerError::from(format!("per-DB resync download failed: {e:?}")))?;

    let status = response.status().as_u16();

    if status == 404 && !is_server_db {
        crate::info!(
            "[{}] per-DB resync: {}/{} not found on leader, removing locally",
            cluster.index,
            owner,
            db,
        );
        let _snapshot_guard = snapshot_lock.write().await;

        match db_pool.delete_db(owner, db).await {
            Ok(()) => {}
            Err(e) if e.status == StatusCode::NOT_FOUND => {}
            Err(e) => return Err(e),
        }
        return Ok(());
    }
    if status != 200 {
        let body = response.text().await.unwrap_or_default();
        return Err(ServerError::from(format!(
            "per-DB resync endpoint returned {status}: {body}",
        )));
    }

    let db_type_str = response
        .headers()
        .get("x-db-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("mapped")
        .to_string();

    let db_type = match db_type_str.as_str() {
        "memory" => DbKind::Memory,
        "file" => DbKind::File,
        _ => DbKind::Mapped,
    };

    let unique_id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!(
        "agdb_resync_{}_{}_{}_{}",
        owner,
        db,
        std::process::id(),
        unique_id
    ));
    std::fs::create_dir_all(&temp_dir)?;
    let _temp_guard = TempDirGuard(temp_dir.clone());
    let data_bin = temp_dir.join("data.bin");

    {
        let mut file = tokio::fs::File::create(&data_bin).await.map_err(|e| {
            ServerError::from(format!("per-DB resync create temp file failed: {e:?}"))
        })?;
        let mut stream = response.bytes_stream();
        let mut total_bytes: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| ServerError::from(format!("per-DB resync stream error: {e:?}")))?;
            total_bytes += chunk.len() as u64;
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                .await
                .map_err(|e| {
                    ServerError::from(format!("per-DB resync write temp file failed: {e:?}"))
                })?;
        }
        tokio::io::AsyncWriteExt::flush(&mut file)
            .await
            .map_err(|e| {
                ServerError::from(format!("per-DB resync flush temp file failed: {e:?}"))
            })?;

        crate::info!(
            "[{}] per-DB resync downloaded: {}/{} ({} bytes)",
            cluster.index,
            owner,
            db,
            total_bytes
        );
    }

    let install_dir = temp_dir.join("install");
    extract_snapshot_binary(
        &data_bin,
        &install_dir,
        config.cluster_max_chunk_size as usize,
    )
    .await?;

    let _snapshot_guard = snapshot_lock.write().await;

    if is_server_db {
        let server_db_src = install_dir.join(SERVER_DB_FILE);
        if !server_db_src.exists() {
            return Err(ServerError::from(
                "per-DB resync: server_db not found in snapshot",
            ));
        }
        let data_dir = std::path::Path::new(&config.data_dir);
        let server_db_dst = data_dir.join(SERVER_DB_FILE);
        let server_db_tmp = data_dir.join(".server_db_resync_tmp");

        // Copy to temp, validate, then rename — so if Db::new fails
        // the original server_db is still intact.
        std::fs::copy(&server_db_src, &server_db_tmp)?;
        let tmp_str = server_db_tmp
            .to_str()
            .ok_or_else(|| ServerError::from("invalid server_db path"))?;
        match agdb::Db::new(tmp_str) {
            Ok(_validated) => {
                let mut db_guard = server_db.db.write().await;
                std::fs::rename(&server_db_tmp, &server_db_dst)?;
                *db_guard = agdb::Db::new(
                    server_db_dst
                        .to_str()
                        .ok_or_else(|| ServerError::from("invalid server_db path"))?,
                )?;
            }
            Err(e) => {
                let _ = std::fs::remove_file(&server_db_tmp);
                return Err(ServerError::from(format!(
                    "per-DB resync: downloaded server_db is invalid: {e}"
                )));
            }
        }
    } else {
        db_pool
            .resync_db(owner, db, db_type, &install_dir, config)
            .await?;
    }

    crate::info!(
        "[{}] per-DB resync complete: {}/{}",
        cluster.index,
        owner,
        db,
    );

    Ok(())
}
