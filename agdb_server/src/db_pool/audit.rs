//! Database audit log stored as one JSON-lines file per UTC day (`YYYY-MM-DD.log`)
//! so that appending never rewrites the log and trimming only removes whole files.

use crate::server_error::ServerError;
use crate::server_error::ServerResult;
use crate::utilities::civil_from_unix_days;
use agdb_api::QueryAudit;
use serde::Deserializer as _;
use serde::de::SeqAccess;
use serde::de::Visitor;
use std::collections::HashSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

const DAY_SECS: u64 = 86_400;
const DAY_FILE_LEN: usize = "YYYY-MM-DD.log".len();
const MIGRATION_DIR: &str = ".migrating";
const LEGACY_FILE: &str = "legacy.log";
const CORRUPT_FILE: &str = "legacy.corrupt";

struct DayWriter<'a> {
    dir: &'a Path,
    day: u64,
    file: Option<BufWriter<File>>,
}

impl<'a> DayWriter<'a> {
    fn new(dir: &'a Path) -> Self {
        Self {
            dir,
            day: 0,
            file: None,
        }
    }

    fn write(&mut self, entry: &QueryAudit) -> ServerResult {
        let day = entry.timestamp / DAY_SECS;

        if self.file.is_none() || self.day != day {
            self.finish()?;
            self.file = Some(BufWriter::new(open_append(
                &self.dir.join(day_file_name(day)),
            )?));
            self.day = day;
        }

        if let Some(file) = &mut self.file {
            serde_json::to_writer(&mut *file, entry)?;
            file.write_all(b"\n")?;
        }

        Ok(())
    }

    fn finish(&mut self) -> ServerResult {
        if let Some(mut file) = self.file.take() {
            file.flush()?;
        }

        Ok(())
    }
}

struct LegacyEntries<'a, 'b> {
    writer: &'a mut DayWriter<'b>,
    error: &'a mut Option<ServerError>,
}

impl<'de> Visitor<'de> for LegacyEntries<'_, '_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an array of audit entries")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while let Some(entry) = seq.next_element::<QueryAudit>()? {
            if let Err(e) = self.writer.write(&entry) {
                let description = e.description.clone();
                *self.error = Some(e);
                return Err(serde::de::Error::custom(description));
            }
        }

        Ok(())
    }
}

pub(crate) fn append(dir: &Path, entries: &[QueryAudit]) -> ServerResult {
    let mut writer = DayWriter::new(dir);

    for entry in entries {
        writer.write(entry)?;
    }

    writer.finish()
}

pub(crate) fn read(dir: &Path) -> ServerResult<Vec<QueryAudit>> {
    let mut audit = vec![];

    for (_, path) in day_files(dir)? {
        let data = match std::fs::read(&path) {
            Ok(data) => data,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };

        for line in lines(&data) {
            match serde_json::from_slice(line) {
                Ok(entry) => audit.push(entry),
                Err(e) => crate::warn!("Skipping invalid audit entry in {}: {e}", path.display()),
            }
        }
    }

    Ok(audit)
}

/// The first UTC day (days since the Unix epoch) kept when keeping
/// today and the `keep_days` days before it at `now` (Unix seconds).
pub(crate) fn first_kept_day(now: u64, keep_days: u64) -> u64 {
    (now / DAY_SECS).saturating_sub(keep_days)
}

/// Removes the day files of the days before `first_day` (days since the Unix epoch).
pub(crate) fn trim(dir: &Path, first_day: u64) -> ServerResult {
    let cutoff = day_file_name(first_day);

    for (name, path) in day_files(dir)? {
        if name < cutoff {
            std::fs::remove_file(&path)?;
        }
    }

    Ok(())
}

/// Makes `dst` hold the same day files as `src`, copying only the days that
/// changed since the last sync. Other files in `dst` are kept.
pub(crate) fn sync(src: &Path, dst: &Path) -> ServerResult {
    let files = day_files(src)?;
    let names = files
        .iter()
        .map(|(name, _)| name.as_str())
        .collect::<HashSet<_>>();

    for (name, path) in day_files(dst)? {
        if !names.contains(name.as_str()) {
            std::fs::remove_file(&path)?;
        }
    }

    if files.is_empty() {
        if std::fs::exists(dst)? && read_entries(dst)?.is_empty() {
            std::fs::remove_dir(dst)?;
        }

        return Ok(());
    }

    std::fs::create_dir_all(dst)?;

    for (name, path) in &files {
        let source = path.metadata()?;
        let modified = source.modified()?;
        let target = dst.join(name);

        if !is_current(&target, source.len(), modified) {
            std::fs::copy(path, &target)?;
            File::options()
                .write(true)
                .open(&target)?
                .set_modified(modified)?;
        }
    }

    Ok(())
}

/// Swaps the audit directories `a` and `b` through `temp`.
pub(crate) fn swap(a: &Path, b: &Path, temp: &Path) -> ServerResult {
    finish_swap(a, b, temp)?;
    move_dir(a, temp)?;
    move_dir(b, a)?;
    move_dir(temp, b)
}

/// Completes a [`swap`] that stopped midway and left `temp` behind.
pub(crate) fn finish_swap(a: &Path, b: &Path, temp: &Path) -> ServerResult {
    if !std::fs::exists(temp)? {
        return Ok(());
    }

    if !std::fs::exists(a)? {
        move_dir(b, a)?;
    }

    if std::fs::exists(b)? {
        return Err(ServerError::from(format!(
            "cannot finish the interrupted audit swap, '{}' and '{}' both exist next to '{}'",
            a.display(),
            b.display(),
            temp.display()
        )));
    }

    move_dir(temp, b)
}

pub(crate) fn rename(src: &Path, dst: &Path) -> ServerResult {
    remove(dst)?;

    if std::fs::exists(src)? {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(src, dst)?;
    }

    Ok(())
}

/// Removes the audit file of the previous format, leaving a directory of the
/// same name (the audit of a database named `{db}.log`) in place.
pub(crate) fn remove_legacy(path: &Path) -> ServerResult {
    if path.is_file() {
        std::fs::remove_file(path)?;
    }

    Ok(())
}

pub(crate) fn remove(path: &Path) -> ServerResult {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => std::fs::remove_dir_all(path)?,
        Ok(_) => std::fs::remove_file(path)?,
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }

    Ok(())
}

/// Converts the audit of the previous format (a single JSON array file) to day
/// files in `dir`, merged in time order with the days already there. The days
/// are staged in `dir` first and the legacy file is moved into the staging
/// directory once they are complete, so an interrupted conversion is either
/// redone or finished on the next call. A legacy file that cannot be fully read
/// is kept as `legacy.corrupt` in `dir` after converting the entries before the
/// error. A write error keeps the legacy file in place.
pub(crate) fn migrate(legacy: &Path, dir: &Path) -> ServerResult {
    let staging = dir.join(MIGRATION_DIR);
    let mut converted = [LEGACY_FILE, CORRUPT_FILE]
        .map(|name| staging.join(name))
        .into_iter()
        .find(|path| path.is_file());

    if converted.is_none() {
        if staging.is_dir() {
            remove(&staging)?;
        }

        if !legacy.is_file() {
            return Ok(());
        }

        let name = if stage_legacy(legacy, dir, &staging)? {
            LEGACY_FILE
        } else {
            CORRUPT_FILE
        };
        let target = staging.join(name);
        std::fs::rename(legacy, &target)?;
        converted = Some(target);
    }

    for (name, path) in day_files(&staging)? {
        std::fs::rename(&path, dir.join(name))?;
    }

    if let Some(corrupt) = converted.filter(|path| path.ends_with(CORRUPT_FILE)) {
        let mut target = dir.join(CORRUPT_FILE);
        let mut n = 0;

        while std::fs::exists(&target)? {
            n += 1;
            target = dir.join(format!("legacy.{n}.corrupt"));
        }

        std::fs::rename(corrupt, target)?;
    }

    remove(&staging)
}

/// Writes the day files of `legacy` merged with those in `dir` to `staging`.
/// Returns `false` if only the entries before an unreadable one were staged.
fn stage_legacy(legacy: &Path, dir: &Path, staging: &Path) -> ServerResult<bool> {
    std::fs::create_dir_all(staging)?;

    let mut writer = DayWriter::new(staging);
    let mut error = None;
    let result = serde_json::Deserializer::from_reader(BufReader::new(File::open(legacy)?))
        .deserialize_seq(LegacyEntries {
            writer: &mut writer,
            error: &mut error,
        });

    if let Some(e) = error {
        return Err(e);
    }

    writer.finish()?;

    for (name, path) in day_files(staging)? {
        let existing = match std::fs::read(dir.join(&name)) {
            Ok(data) => data,
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        let staged = std::fs::read(&path)?;
        std::fs::write(&path, merge(&staged, &existing))?;
    }

    if let Err(e) = result {
        crate::warn!(
            "Audit '{}' is not fully readable, converted the entries before the error and kept the file as '{CORRUPT_FILE}': {e}",
            legacy.display()
        );
        return Ok(false);
    }

    Ok(true)
}

/// Merges the lines of two day files ordered by their timestamps, `first`
/// before `second` on equal ones. A line without a readable timestamp stays
/// after the line before it.
fn merge(first: &[u8], second: &[u8]) -> Vec<u8> {
    #[derive(serde::Deserialize)]
    struct Timestamp {
        timestamp: u64,
    }

    let mut entries = vec![];

    for data in [first, second] {
        let mut timestamp = 0;

        for line in lines(data) {
            if let Ok(t) = serde_json::from_slice::<Timestamp>(line) {
                timestamp = t.timestamp;
            }

            entries.push((timestamp, line));
        }
    }

    entries.sort_by_key(|(timestamp, _)| *timestamp);
    let mut merged = Vec::with_capacity(first.len() + second.len() + 1);

    for (_, line) in entries {
        merged.extend_from_slice(line);
        merged.push(b'\n');
    }

    merged
}

fn move_dir(src: &Path, dst: &Path) -> ServerResult {
    if std::fs::exists(src)? {
        if let Some(parent) = dst.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(src, dst)?;
    }

    Ok(())
}

fn open_append(path: &Path) -> ServerResult<File> {
    let open = || {
        OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)
    };

    let mut file = match open() {
        Err(e) if e.kind() == ErrorKind::NotFound => {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            open()?
        }
        file => file?,
    };

    if file.metadata()?.len() > 0 {
        let mut last = [0_u8];
        file.seek(SeekFrom::End(-1))?;
        file.read_exact(&mut last)?;

        if last[0] != b'\n' {
            file.write_all(b"\n")?;
        }
    }

    Ok(file)
}

fn is_current(target: &Path, len: u64, modified: SystemTime) -> bool {
    target
        .metadata()
        .is_ok_and(|t| t.len() == len && t.modified().is_ok_and(|m| m == modified))
}

fn day_file_name(day: u64) -> String {
    let (year, month, day) = civil_from_unix_days(day);
    format!("{year:04}-{month:02}-{day:02}.log")
}

fn is_day_file(name: &str) -> bool {
    name.len() == DAY_FILE_LEN
        && name.ends_with(".log")
        && name.bytes().take(10).enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        })
}

fn day_files(dir: &Path) -> ServerResult<Vec<(String, PathBuf)>> {
    let mut files = read_entries(dir)?
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_string_lossy().to_string();
            is_day_file(&name).then_some((name, path))
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn read_entries(dir: &Path) -> ServerResult<Vec<PathBuf>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.into()),
    };

    let mut paths = vec![];

    for entry in entries {
        paths.push(entry?.path());
    }

    Ok(paths)
}

fn lines(data: &[u8]) -> impl Iterator<Item = &[u8]> {
    data.split(|b| *b == b'\n').filter(|line| !line.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use agdb::QueryBuilder;

    const DAY_2020_01_01: u64 = 1_577_836_800;
    const DAY_2020_01_02: u64 = DAY_2020_01_01 + DAY_SECS;
    const DAY_2020_01_03: u64 = DAY_2020_01_02 + DAY_SECS;

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "agdb_audit_{name}_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn entry(timestamp: u64) -> QueryAudit {
        QueryAudit {
            timestamp,
            username: "user".to_string(),
            query: QueryBuilder::insert().nodes().count(1).query().into(),
        }
    }

    fn timestamps(dir: &Path) -> Vec<u64> {
        read(dir).unwrap().iter().map(|e| e.timestamp).collect()
    }

    fn names(dir: &Path) -> Vec<String> {
        let mut names = read_entries(dir)
            .unwrap()
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        names.sort();
        names
    }

    #[test]
    fn append_writes_one_file_per_day() {
        let test_dir = TestDir::new("append");
        let dir = test_dir.0.join("db");

        append(&dir, &[entry(DAY_2020_01_01), entry(DAY_2020_01_02 - 1)]).unwrap();
        append(&dir, &[entry(DAY_2020_01_02)]).unwrap();

        assert_eq!(names(&dir), ["2020-01-01.log", "2020-01-02.log"]);
        assert_eq!(
            timestamps(&dir),
            [DAY_2020_01_01, DAY_2020_01_02 - 1, DAY_2020_01_02]
        );
    }

    #[test]
    fn append_after_partial_line_starts_a_new_line() {
        let test_dir = TestDir::new("partial");
        let dir = test_dir.0.join("db");
        append(&dir, &[entry(DAY_2020_01_01)]).unwrap();
        let file = dir.join("2020-01-01.log");
        let mut data = std::fs::read(&file).unwrap();
        data.extend_from_slice(b"{\"timest");
        std::fs::write(&file, data).unwrap();

        append(&dir, &[entry(DAY_2020_01_01 + 1)]).unwrap();

        assert_eq!(timestamps(&dir), [DAY_2020_01_01, DAY_2020_01_01 + 1]);
    }

    #[test]
    fn read_missing_dir_is_empty() {
        let test_dir = TestDir::new("read_missing");

        assert!(read(&test_dir.0.join("db")).unwrap().is_empty());
    }

    #[test]
    fn read_ignores_other_files() {
        let test_dir = TestDir::new("read_other");
        let dir = test_dir.0.join("db");
        append(&dir, &[entry(DAY_2020_01_01)]).unwrap();
        std::fs::write(dir.join("notes.log"), "[]").unwrap();
        std::fs::write(dir.join("2020-01-02.tmp"), "{}").unwrap();

        assert_eq!(timestamps(&dir), [DAY_2020_01_01]);
    }

    #[test]
    fn trim_removes_the_days_before_the_first_day() {
        let test_dir = TestDir::new("trim");
        let dir = test_dir.0.join("db");
        append(
            &dir,
            &[
                entry(DAY_2020_01_01),
                entry(DAY_2020_01_02),
                entry(DAY_2020_01_02 + 10),
                entry(DAY_2020_01_03),
            ],
        )
        .unwrap();
        let modified = dir
            .join("2020-01-02.log")
            .metadata()
            .unwrap()
            .modified()
            .unwrap();

        trim(&dir, DAY_2020_01_02 / DAY_SECS).unwrap();

        assert_eq!(names(&dir), ["2020-01-02.log", "2020-01-03.log"]);
        assert_eq!(
            timestamps(&dir),
            [DAY_2020_01_02, DAY_2020_01_02 + 10, DAY_2020_01_03]
        );
        assert_eq!(
            dir.join("2020-01-02.log")
                .metadata()
                .unwrap()
                .modified()
                .unwrap(),
            modified
        );
    }

    #[test]
    fn trim_keeps_other_files() {
        let test_dir = TestDir::new("trim_other");
        let dir = test_dir.0.join("db");
        append(&dir, &[entry(DAY_2020_01_01)]).unwrap();
        std::fs::write(dir.join("notes.log"), "").unwrap();

        trim(&dir, DAY_2020_01_03 / DAY_SECS).unwrap();

        assert_eq!(names(&dir), ["notes.log"]);
    }

    #[test]
    fn first_kept_day_counts_back_from_today() {
        let now = DAY_2020_01_03 + 10;

        assert_eq!(first_kept_day(now, 0), DAY_2020_01_03 / DAY_SECS);
        assert_eq!(first_kept_day(now, 1), DAY_2020_01_02 / DAY_SECS);
        assert_eq!(
            first_kept_day(DAY_2020_01_03 - 1, 1),
            DAY_2020_01_01 / DAY_SECS
        );
        assert_eq!(first_kept_day(now, u64::MAX), 0);
    }

    #[test]
    fn trim_missing_dir() {
        let test_dir = TestDir::new("trim_missing");

        trim(&test_dir.0.join("db"), DAY_2020_01_02 / DAY_SECS).unwrap();
    }

    #[test]
    fn sync_copies_changed_days_and_removes_others() {
        let test_dir = TestDir::new("sync");
        let src = test_dir.0.join("src");
        let dst = test_dir.0.join("dst");
        append(&src, &[entry(DAY_2020_01_01), entry(DAY_2020_01_02)]).unwrap();
        sync(&src, &dst).unwrap();
        append(&src, &[entry(DAY_2020_01_02 + 1)]).unwrap();
        trim(&src, DAY_2020_01_02 / DAY_SECS).unwrap();
        std::fs::write(dst.join("2020-01-03.log"), "").unwrap();

        sync(&src, &dst).unwrap();

        assert_eq!(names(&dst), ["2020-01-02.log"]);
        assert_eq!(timestamps(&dst), [DAY_2020_01_02, DAY_2020_01_02 + 1]);
    }

    #[test]
    fn sync_skips_unchanged_days() {
        let test_dir = TestDir::new("sync_unchanged");
        let src = test_dir.0.join("src");
        let dst = test_dir.0.join("dst");
        append(&src, &[entry(DAY_2020_01_01)]).unwrap();
        sync(&src, &dst).unwrap();
        let modified = dst
            .join("2020-01-01.log")
            .metadata()
            .unwrap()
            .modified()
            .unwrap();

        sync(&src, &dst).unwrap();

        assert_eq!(
            dst.join("2020-01-01.log")
                .metadata()
                .unwrap()
                .modified()
                .unwrap(),
            modified
        );
    }

    #[test]
    fn sync_empty_source_removes_target() {
        let test_dir = TestDir::new("sync_empty");
        let dst = test_dir.0.join("dst");
        append(&dst, &[entry(DAY_2020_01_01)]).unwrap();

        sync(&test_dir.0.join("src"), &dst).unwrap();

        assert!(!dst.exists());
    }

    #[test]
    fn sync_copies_a_rewritten_day_of_the_same_size() {
        let test_dir = TestDir::new("sync_rewritten");
        let src = test_dir.0.join("src");
        let dst = test_dir.0.join("dst");
        append(&src, &[entry(DAY_2020_01_01)]).unwrap();
        sync(&src, &dst).unwrap();
        let file = src.join("2020-01-01.log");
        std::fs::remove_file(&file).unwrap();
        append(&src, &[entry(DAY_2020_01_01 + 1)]).unwrap();
        File::options()
            .write(true)
            .open(&file)
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH)
            .unwrap();

        sync(&src, &dst).unwrap();

        assert_eq!(timestamps(&dst), [DAY_2020_01_01 + 1]);
    }

    #[test]
    fn sync_replaces_target_days_and_keeps_other_files() {
        let test_dir = TestDir::new("sync_replace");
        let src = test_dir.0.join("src");
        let dst = test_dir.0.join("dst");
        append(&src, &[entry(DAY_2020_01_01)]).unwrap();
        append(&dst, &[entry(DAY_2020_01_01 + 1), entry(DAY_2020_01_02)]).unwrap();
        File::options()
            .write(true)
            .open(dst.join("2020-01-01.log"))
            .unwrap()
            .set_modified(SystemTime::UNIX_EPOCH)
            .unwrap();
        std::fs::write(dst.join(CORRUPT_FILE), "[").unwrap();

        sync(&src, &dst).unwrap();

        assert_eq!(names(&dst), ["2020-01-01.log", CORRUPT_FILE]);
        assert_eq!(timestamps(&dst), [DAY_2020_01_01]);
    }

    #[test]
    fn swap_exchanges_the_dirs() {
        let test_dir = TestDir::new("swap");
        let a = test_dir.0.join("a");
        let b = test_dir.0.join("b");
        let temp = test_dir.0.join("swap").join("a");
        append(&a, &[entry(DAY_2020_01_01)]).unwrap();
        append(&b, &[entry(DAY_2020_01_02)]).unwrap();

        swap(&a, &b, &temp).unwrap();

        assert_eq!(timestamps(&a), [DAY_2020_01_02]);
        assert_eq!(timestamps(&b), [DAY_2020_01_01]);
        assert!(!temp.exists());
    }

    #[test]
    fn swap_one_side_missing() {
        let test_dir = TestDir::new("swap_missing");
        let a = test_dir.0.join("a");
        let b = test_dir.0.join("b");
        let temp = test_dir.0.join("temp");
        append(&a, &[entry(DAY_2020_01_01)]).unwrap();

        swap(&a, &b, &temp).unwrap();

        assert!(!a.exists());
        assert_eq!(timestamps(&b), [DAY_2020_01_01]);
    }

    #[test]
    fn finish_swap_after_the_first_move() {
        let test_dir = TestDir::new("finish_swap_first");
        let a = test_dir.0.join("a");
        let b = test_dir.0.join("b");
        let temp = test_dir.0.join("temp");
        append(&temp, &[entry(DAY_2020_01_01)]).unwrap();
        append(&b, &[entry(DAY_2020_01_02)]).unwrap();

        finish_swap(&a, &b, &temp).unwrap();

        assert_eq!(timestamps(&a), [DAY_2020_01_02]);
        assert_eq!(timestamps(&b), [DAY_2020_01_01]);
        assert!(!temp.exists());
    }

    #[test]
    fn finish_swap_after_the_second_move() {
        let test_dir = TestDir::new("finish_swap_second");
        let a = test_dir.0.join("a");
        let b = test_dir.0.join("b");
        let temp = test_dir.0.join("temp");
        append(&temp, &[entry(DAY_2020_01_01)]).unwrap();
        append(&a, &[entry(DAY_2020_01_02)]).unwrap();

        finish_swap(&a, &b, &temp).unwrap();

        assert_eq!(timestamps(&a), [DAY_2020_01_02]);
        assert_eq!(timestamps(&b), [DAY_2020_01_01]);
    }

    #[test]
    fn finish_swap_keeps_everything_when_ambiguous() {
        let test_dir = TestDir::new("finish_swap_ambiguous");
        let a = test_dir.0.join("a");
        let b = test_dir.0.join("b");
        let temp = test_dir.0.join("temp");
        append(&temp, &[entry(DAY_2020_01_01)]).unwrap();
        append(&a, &[entry(DAY_2020_01_02)]).unwrap();
        append(&b, &[entry(DAY_2020_01_03)]).unwrap();

        assert!(swap(&a, &b, &temp).is_err());

        assert_eq!(timestamps(&temp), [DAY_2020_01_01]);
        assert_eq!(timestamps(&a), [DAY_2020_01_02]);
        assert_eq!(timestamps(&b), [DAY_2020_01_03]);
    }

    #[test]
    fn remove_legacy_keeps_a_dir() {
        let test_dir = TestDir::new("remove_legacy");
        let file = test_dir.0.join("db.log");
        let dir = test_dir.0.join("db.log.log");
        std::fs::write(&file, "[]").unwrap();
        append(&dir, &[entry(DAY_2020_01_01)]).unwrap();

        remove_legacy(&file).unwrap();
        remove_legacy(&dir).unwrap();
        remove_legacy(&test_dir.0.join("missing")).unwrap();

        assert!(!file.exists());
        assert_eq!(timestamps(&dir), [DAY_2020_01_01]);
    }

    #[test]
    fn rename_replaces_target() {
        let test_dir = TestDir::new("rename");
        let src = test_dir.0.join("src");
        let dst = test_dir.0.join("other").join("dst");
        append(&src, &[entry(DAY_2020_01_01)]).unwrap();
        append(&dst, &[entry(DAY_2020_01_02)]).unwrap();

        rename(&src, &dst).unwrap();

        assert!(!src.exists());
        assert_eq!(timestamps(&dst), [DAY_2020_01_01]);
    }

    #[test]
    fn rename_missing_source_removes_target() {
        let test_dir = TestDir::new("rename_missing");
        let dst = test_dir.0.join("dst");
        append(&dst, &[entry(DAY_2020_01_02)]).unwrap();

        rename(&test_dir.0.join("src"), &dst).unwrap();

        assert!(!dst.exists());
    }

    #[test]
    fn remove_file_dir_and_missing() {
        let test_dir = TestDir::new("remove");
        let file = test_dir.0.join("file");
        let dir = test_dir.0.join("dir");
        std::fs::write(&file, "").unwrap();
        append(&dir, &[entry(DAY_2020_01_01)]).unwrap();

        remove(&file).unwrap();
        remove(&dir).unwrap();
        remove(&test_dir.0.join("missing")).unwrap();

        assert!(!file.exists());
        assert!(!dir.exists());
    }

    #[test]
    fn migrate_legacy_array() {
        let test_dir = TestDir::new("migrate");
        let legacy = test_dir.0.join("db.log");
        let dir = test_dir.0.join("db");
        let entries = [
            entry(DAY_2020_01_01),
            entry(DAY_2020_01_02),
            entry(DAY_2020_01_02 + 1),
        ];
        std::fs::write(&legacy, serde_json::to_vec(&entries).unwrap()).unwrap();

        migrate(&legacy, &dir).unwrap();

        assert!(!legacy.exists());
        assert_eq!(names(&dir), ["2020-01-01.log", "2020-01-02.log"]);
        assert_eq!(read(&dir).unwrap(), entries);
        assert_eq!(names(&test_dir.0), ["db"]);
    }

    #[test]
    fn migrate_truncated_legacy_keeps_the_readable_entries() {
        let test_dir = TestDir::new("migrate_truncated");
        let legacy = test_dir.0.join("db.log");
        let dir = test_dir.0.join("db");
        let mut data = serde_json::to_vec(&[entry(DAY_2020_01_01)]).unwrap();
        data.pop();
        data.extend_from_slice(b",{\"timestamp\":");
        std::fs::write(&legacy, data).unwrap();

        let data = std::fs::read(&legacy).unwrap();

        migrate(&legacy, &dir).unwrap();

        assert!(!legacy.exists());
        assert_eq!(timestamps(&dir), [DAY_2020_01_01]);
        assert_eq!(names(&dir), ["2020-01-01.log", CORRUPT_FILE]);
        assert_eq!(std::fs::read(dir.join(CORRUPT_FILE)).unwrap(), data);
    }

    #[test]
    fn migrate_keeps_an_earlier_corrupt_file() {
        let test_dir = TestDir::new("migrate_corrupt_twice");
        let legacy = test_dir.0.join("db.log");
        let dir = test_dir.0.join("db");
        std::fs::write(&legacy, "[").unwrap();
        migrate(&legacy, &dir).unwrap();
        std::fs::write(&legacy, "[{").unwrap();

        migrate(&legacy, &dir).unwrap();

        assert_eq!(names(&dir), ["legacy.1.corrupt", CORRUPT_FILE]);
    }

    #[test]
    fn migrate_into_existing_dir_merges_in_time_order() {
        let test_dir = TestDir::new("migrate_existing");
        let legacy = test_dir.0.join("db.log");
        let dir = test_dir.0.join("db");
        append(&dir, &[entry(DAY_2020_01_02 + 5)]).unwrap();
        std::fs::write(
            &legacy,
            serde_json::to_vec(&[entry(DAY_2020_01_01), entry(DAY_2020_01_02 + 1)]).unwrap(),
        )
        .unwrap();

        migrate(&legacy, &dir).unwrap();

        assert_eq!(
            timestamps(&dir),
            [DAY_2020_01_01, DAY_2020_01_02 + 1, DAY_2020_01_02 + 5]
        );
        assert_eq!(names(&test_dir.0), ["db"]);
        assert_eq!(names(&dir), ["2020-01-01.log", "2020-01-02.log"]);
    }

    #[test]
    fn migrate_finishes_a_committed_conversion() {
        let test_dir = TestDir::new("migrate_committed");
        let legacy = test_dir.0.join("db.log");
        let dir = test_dir.0.join("db");
        let staging = dir.join(MIGRATION_DIR);
        append(&dir, &[entry(DAY_2020_01_01)]).unwrap();
        append(&staging, &[entry(DAY_2020_01_02)]).unwrap();
        std::fs::write(staging.join(LEGACY_FILE), "[]").unwrap();

        migrate(&legacy, &dir).unwrap();

        assert_eq!(timestamps(&dir), [DAY_2020_01_01, DAY_2020_01_02]);
        assert_eq!(names(&dir), ["2020-01-01.log", "2020-01-02.log"]);
    }

    #[test]
    fn migrate_redoes_an_uncommitted_conversion() {
        let test_dir = TestDir::new("migrate_uncommitted");
        let legacy = test_dir.0.join("db.log");
        let dir = test_dir.0.join("db");
        append(&dir.join(MIGRATION_DIR), &[entry(DAY_2020_01_01)]).unwrap();
        std::fs::write(
            &legacy,
            serde_json::to_vec(&[entry(DAY_2020_01_01 + 1)]).unwrap(),
        )
        .unwrap();

        migrate(&legacy, &dir).unwrap();

        assert_eq!(timestamps(&dir), [DAY_2020_01_01 + 1]);
        assert_eq!(names(&dir), ["2020-01-01.log"]);
    }

    #[test]
    fn migrate_removes_a_stale_staging_without_legacy_file() {
        let test_dir = TestDir::new("migrate_stale");
        let dir = test_dir.0.join("db");
        append(&dir.join(MIGRATION_DIR), &[entry(DAY_2020_01_01)]).unwrap();

        migrate(&test_dir.0.join("db.log"), &dir).unwrap();

        assert!(names(&dir).is_empty());
    }

    #[test]
    fn migrate_skips_a_dir_named_like_the_legacy_file() {
        let test_dir = TestDir::new("migrate_dir_named_log");
        let legacy = test_dir.0.join("db.log");
        append(&legacy, &[entry(DAY_2020_01_01)]).unwrap();

        migrate(&legacy, &test_dir.0.join("db")).unwrap();

        assert_eq!(timestamps(&legacy), [DAY_2020_01_01]);
        assert_eq!(names(&test_dir.0), ["db.log"]);
    }

    #[test]
    fn merge_orders_by_timestamp() {
        let line = |t: u64| serde_json::to_string(&entry(t)).unwrap();
        let first = format!("{}\n{}\n", line(1), line(3));
        let second = format!("{}\n{{bad\n{}\n", line(2), line(3));

        let merged = String::from_utf8(merge(first.as_bytes(), second.as_bytes())).unwrap();

        assert_eq!(
            merged,
            format!(
                "{}\n{}\n{{bad\n{}\n{}\n",
                line(1),
                line(2),
                line(3),
                line(3)
            )
        );
    }

    #[test]
    fn migrate_without_legacy_file() {
        let test_dir = TestDir::new("migrate_none");
        let dir = test_dir.0.join("db");

        migrate(&test_dir.0.join("db.log"), &dir).unwrap();

        assert!(!dir.exists());
    }

    #[test]
    fn migrate_into_unwritable_dir_keeps_legacy_file() {
        let test_dir = TestDir::new("migrate_unwritable");
        let legacy = test_dir.0.join("db.log");
        std::fs::write(
            &legacy,
            serde_json::to_vec(&[entry(DAY_2020_01_01)]).unwrap(),
        )
        .unwrap();
        std::fs::write(test_dir.0.join("file"), "").unwrap();

        assert!(migrate(&legacy, &test_dir.0.join("file").join("db")).is_err());
        assert!(legacy.exists());
    }

    #[test]
    fn legacy_entries_report_write_error() {
        let test_dir = TestDir::new("legacy_error");
        std::fs::create_dir_all(test_dir.0.join("2020-01-01.log")).unwrap();
        let data = serde_json::to_vec(&[entry(DAY_2020_01_01)]).unwrap();
        let mut writer = DayWriter::new(&test_dir.0);
        let mut error = None;

        let result = serde_json::Deserializer::from_slice(&data).deserialize_seq(LegacyEntries {
            writer: &mut writer,
            error: &mut error,
        });

        assert!(result.is_err());
        assert!(error.is_some());
    }

    #[test]
    fn day_file_names() {
        assert_eq!(day_file_name(0), "1970-01-01.log");
        assert_eq!(day_file_name(DAY_2020_01_01 / DAY_SECS), "2020-01-01.log");
        assert!(is_day_file("2020-01-01.log"));
        assert!(!is_day_file("2020-01-01.tmp"));
        assert!(!is_day_file("2020_01-01.log"));
        assert!(!is_day_file("2020-0a-01.log"));
        assert!(!is_day_file("db.log"));
    }
}
