use crate::DbError;
use agplatform::Fs;
use agplatform::OpenWriteOptions;
use agplatform::Platform;
use agplatform::ReadFile;
use agplatform::WriteFile;

type FileHandle<P> = <<P as Platform>::Fs as Fs>::WriteHandle;

/// A log of a version without the header starts with the position of a
/// write, which is never 1 as the storage writes below the end of its
/// version record only at 0 and 16.
const VERSION: u64 = 1;
/// A version without the header reads the generation as the size of a
/// record at 1, which this runs past the end of the log, so that version
/// discards the log rather than truncating the file to 1 byte.
const FIRST_GENERATION: u64 = 1 << 62;
const GENERATION_POS: u64 = 8;
const HEADER_SIZE: u64 = 16;
const TRUNCATE_SIZE: u64 = 1024 * 1024;

/// The log is `[VERSION][generation]` followed by the records
/// `[pos][size][value][generation][end]`, `end` being the position the
/// record ends at. The records are written over those of the previous
/// generations, clearing the log bumps the generation and truncates the
/// file only once it grows over `TRUNCATE_SIZE`. The valid records are
/// those up to the first one running past the end of the file or not
/// ending with the generation and its own end, i.e. stale or partially
/// written. An empty file is an empty log, its header is written with the
/// first record. The log of a version without the header holds just the
/// records and is truncated on clear. Truncating makes the following
/// writes slower as they grow the file again, but leaves nothing for a
/// following sync to write.
pub struct WriteAheadLog<P: Platform> {
    file: FileHandle<P>,
    file_len: u64,
    /// `None` without the header.
    generation: Option<u64>,
    len: u64,
    staged: Vec<u8>,
}

impl<P: Platform> std::fmt::Debug for WriteAheadLog<P> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WriteAheadLog")
            .field("generation", &self.generation)
            .field("len", &self.len)
            .finish_non_exhaustive()
    }
}

impl<P: Platform> WriteAheadLog<P> {
    pub fn clear(&mut self) -> Result<(), DbError> {
        if self.is_empty() {
            return Ok(());
        }

        match self.generation {
            Some(generation) if self.file_len <= TRUNCATE_SIZE => {
                let generation = generation.wrapping_add(1);
                self.file
                    .write_all_at(&generation.to_le_bytes(), GENERATION_POS)?;
                self.generation = Some(generation);
                self.len = HEADER_SIZE;
            }
            _ => self.truncate()?,
        }

        Ok(())
    }

    pub fn discard_staged(&mut self) {
        self.staged.clear();
    }

    pub fn sync(&mut self) -> Result<(), DbError> {
        Ok(self.file.sync_data()?)
    }

    pub fn is_empty(&self) -> bool {
        self.len == self.start()
    }

    pub fn new(platform: &P, filename: &str) -> Result<Self, DbError> {
        let file = platform
            .fs()
            .open_write_options(wal_filename(filename), OpenWriteOptions::new().create())?;
        let mut bytes = vec![0_u8; file.metadata()?.len as usize];
        file.read_exact_at(&mut bytes, 0)?;
        let generation = generation(&bytes);
        let start = generation.map_or(0, |_| HEADER_SIZE);
        let mut records = Records::new(&bytes[start as usize..], start, generation);
        records.by_ref().count();
        let len = records.offset;

        Ok(Self {
            file,
            file_len: bytes.len() as u64,
            generation,
            len,
            staged: Vec::new(),
        })
    }

    /// Calls `undo` with the position and value of each record from the
    /// newest to the oldest.
    pub fn rollback(
        &self,
        mut undo: impl FnMut(u64, &[u8]) -> Result<(), DbError>,
    ) -> Result<(), DbError> {
        let start = self.start();
        let mut bytes = vec![0_u8; (self.len - start) as usize];
        self.file.read_exact_at(&mut bytes, start)?;
        let records: Vec<(u64, &[u8])> = Records::new(&bytes, start, self.generation).collect();

        for (pos, value) in records.into_iter().rev() {
            undo(pos, value)?;
        }

        Ok(())
    }

    pub fn stage(&mut self, pos: u64, value: &[u8]) {
        let value_start = self.stage_record(pos, value.len() as u64);
        self.staged[value_start..][..value.len()].copy_from_slice(value);
    }

    pub fn stage_with(
        &mut self,
        pos: u64,
        size: u64,
        value: &dyn Fn(&mut [u8]) -> Result<(), DbError>,
    ) -> Result<&[u8], DbError> {
        let staged_len = self.staged.len();
        let value_start = self.stage_record(pos, size);
        let staged = &mut self.staged[value_start..][..size as usize];

        if let Err(error) = value(staged) {
            self.staged.truncate(staged_len);
            return Err(error);
        }

        Ok(&self.staged[value_start..][..size as usize])
    }

    pub fn staged_len(&self) -> u64 {
        self.staged.len() as u64
    }

    pub fn truncate(&mut self) -> Result<(), DbError> {
        self.file.set_len(0)?;
        self.file_len = 0;
        self.generation = None;
        self.len = 0;
        self.staged.clear();
        Ok(())
    }

    pub fn write_staged(&mut self) -> Result<(), DbError> {
        if self.staged.is_empty() {
            return Ok(());
        }

        let end = self.len + self.staged.len() as u64;
        let written = self.file.write_all_at(&self.staged, self.len);
        self.file_len = self.file_len.max(end);
        written?;
        self.generation = self.next_generation();
        self.len = end;
        self.staged.clear();
        Ok(())
    }

    fn stage_record(&mut self, pos: u64, size: u64) -> usize {
        if self.len + self.staged.len() as u64 == 0 {
            self.staged.extend_from_slice(&header(FIRST_GENERATION));
        }

        let generation = self.next_generation();
        append_record(&mut self.staged, self.len, pos, size, generation)
    }

    fn next_generation(&self) -> Option<u64> {
        if self.len == 0 {
            Some(FIRST_GENERATION)
        } else {
            self.generation
        }
    }

    fn start(&self) -> u64 {
        self.generation.map_or(0, |_| HEADER_SIZE)
    }
}

/// Truncates the empty log on a clean close so that a version without
/// the header does not find it.
impl<P: Platform> Drop for WriteAheadLog<P> {
    fn drop(&mut self) {
        if self.is_empty() && self.file_len != 0 {
            let _ = self.file.set_len(0);
        }
    }
}

/// The records of the `bytes` read from the `offset` of the log, without
/// trailers if the `generation` is `None`.
struct Records<'a> {
    bytes: &'a [u8],
    offset: u64,
    generation: Option<u64>,
}

impl<'a> Records<'a> {
    fn new(bytes: &'a [u8], offset: u64, generation: Option<u64>) -> Self {
        Self {
            bytes,
            offset,
            generation,
        }
    }
}

impl<'a> Iterator for Records<'a> {
    type Item = (u64, &'a [u8]);

    fn next(&mut self) -> Option<Self::Item> {
        let (pos, bytes) = self.bytes.split_first_chunk::<8>()?;
        let (size, bytes) = bytes.split_first_chunk::<8>()?;
        let size = usize::try_from(u64::from_le_bytes(*size)).unwrap_or(usize::MAX);
        let (value, mut bytes) = bytes.split_at_checked(size)?;

        if let Some(generation) = self.generation {
            let (record_generation, rest) = bytes.split_first_chunk::<8>()?;
            let (end, rest) = rest.split_first_chunk::<8>()?;
            let expected_end = self.offset + (self.bytes.len() - rest.len()) as u64;

            if u64::from_le_bytes(*record_generation) != generation
                || u64::from_le_bytes(*end) != expected_end
            {
                return None;
            }

            bytes = rest;
        }

        self.offset += (self.bytes.len() - bytes.len()) as u64;
        self.bytes = bytes;
        Some((u64::from_le_bytes(*pos), value))
    }
}

fn generation(bytes: &[u8]) -> Option<u64> {
    let (version, bytes) = bytes.split_first_chunk::<8>()?;
    let (generation, _) = bytes.split_first_chunk::<8>()?;
    (u64::from_le_bytes(*version) == VERSION).then(|| u64::from_le_bytes(*generation))
}

fn header(generation: u64) -> [u8; HEADER_SIZE as usize] {
    let mut header = [0_u8; HEADER_SIZE as usize];
    header[..8].copy_from_slice(&VERSION.to_le_bytes());
    header[8..].copy_from_slice(&generation.to_le_bytes());
    header
}

fn append_record(
    bytes: &mut Vec<u8>,
    offset: u64,
    pos: u64,
    size: u64,
    generation: Option<u64>,
) -> usize {
    bytes.extend_from_slice(&pos.to_le_bytes());
    bytes.extend_from_slice(&size.to_le_bytes());
    let value_start = bytes.len();
    bytes.resize(value_start + size as usize, 0);

    if let Some(generation) = generation {
        bytes.extend_from_slice(&generation.to_le_bytes());
        let end = offset + bytes.len() as u64 + 8;
        bytes.extend_from_slice(&end.to_le_bytes());
    }

    value_start
}

pub(crate) fn remove_wal<P: Platform>(platform: &P, filename: &str) -> Result<(), DbError> {
    let name = wal_filename(filename);

    if platform.fs().exists(&name)? {
        platform.fs().remove_file(&name)?;
    }

    Ok(())
}

pub(crate) fn wal_filename(filename: &str) -> String {
    let pos;

    if let Some(slash) = filename.rfind('/') {
        pos = slash + 1;
    } else if let Some(backslash) = filename.rfind('\\') {
        pos = backslash + 1
    } else {
        pos = 0;
    }

    let mut name = filename.to_owned();
    name.insert(pos, '.');

    name
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::test_platform::TestPlatform;
    use agplatform::test_platform::test_fs::FsOp;
    use agplatform::test_platform::test_platform;
    use std::io::ErrorKind;

    const NAME: &str = "/db";
    const WAL: &str = "/.db";

    fn legacy_record(pos: u64, value: &[u8]) -> Vec<u8> {
        [
            &pos.to_le_bytes(),
            &(value.len() as u64).to_le_bytes(),
            value,
        ]
        .concat()
    }

    fn record_at(offset: u64, pos: u64, value: &[u8], generation: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        let value_start = append_record(
            &mut bytes,
            offset,
            pos,
            value.len() as u64,
            Some(generation),
        );
        bytes[value_start..][..value.len()].copy_from_slice(value);
        bytes
    }

    fn open(platform: &TestPlatform) -> WriteAheadLog<TestPlatform> {
        WriteAheadLog::new(platform, NAME).unwrap()
    }

    fn undone(wal: &WriteAheadLog<TestPlatform>) -> Result<Vec<(u64, Vec<u8>)>, DbError> {
        let mut undone = Vec::new();
        wal.rollback(|pos, value| {
            undone.push((pos, value.to_vec()));
            Ok(())
        })?;
        Ok(undone)
    }

    fn records(wal: &WriteAheadLog<TestPlatform>) -> Vec<(u64, Vec<u8>)> {
        let mut records = undone(wal).unwrap();
        records.reverse();
        records
    }

    fn kill(platform: &TestPlatform) {
        platform.fs().sync();
        platform.fs().crash();
    }

    fn insert(
        wal: &mut WriteAheadLog<TestPlatform>,
        pos: u64,
        value: &[u8],
    ) -> Result<(), DbError> {
        wal.stage(pos, value);
        let written = wal.write_staged();
        wal.discard_staged();
        written
    }

    fn wal_bytes(platform: &TestPlatform) -> Vec<u8> {
        platform.fs().read(WAL).unwrap()
    }

    fn wal_len(platform: &TestPlatform) -> u64 {
        platform.fs().metadata(WAL).unwrap().len
    }

    fn tear_write(platform: &TestPlatform, wal: &mut WriteAheadLog<TestPlatform>) {
        platform.fs().set_short_io(FsOp::Write, WAL, 50);
        platform.fs().set_capacity(Some(wal.len + 50));
        assert!(insert(wal, 9, &[9; 100]).is_err());
        assert_eq!(wal_len(platform), wal.len + 50);
        platform.fs().set_capacity(None);
        platform.fs().clear_faults();
    }

    mod write_ahead_log {
        use super::*;

        mod std_fmt_debug {
            use super::*;

            #[test]
            fn fmt_should_show_generation_and_len() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();

                assert_eq!(
                    format!("{wal:?}"),
                    "WriteAheadLog { generation: Some(4611686018427387904), len: 50, .. }"
                );
            }
        }

        mod clear {
            use super::*;

            #[test]
            fn clear_should_invalidate_records_by_bumping_generation() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();
                insert(&mut wal, 2, &[3]).unwrap();

                wal.clear().unwrap();

                assert!(wal.is_empty());
                assert!(records(&wal).is_empty());
                assert_eq!(wal_len(&platform), 16 + 34 + 33);
                kill(&platform);
                assert_eq!(
                    generation(&wal_bytes(&platform)),
                    Some(FIRST_GENERATION + 1)
                );
                assert!(records(&open(&platform)).is_empty());
            }

            #[test]
            fn clear_should_let_records_overwrite_invalidated_ones() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2, 3, 4]).unwrap();
                insert(&mut wal, 2, &[5]).unwrap();
                wal.clear().unwrap();

                insert(&mut wal, 3, &[6]).unwrap();

                assert_eq!(records(&wal), vec![(3, vec![6])]);
                assert_eq!(wal_len(&platform), 16 + 36 + 33);
                kill(&platform);
                assert_eq!(records(&open(&platform)), vec![(3, vec![6])]);
            }

            #[test]
            fn clear_empty_should_do_nothing() {
                let platform = test_platform();
                let mut wal = open(&platform);
                platform
                    .fs()
                    .set_io_error(FsOp::Write, WAL, ErrorKind::Other);
                platform
                    .fs()
                    .set_io_error(FsOp::SetLen, WAL, ErrorKind::Other);

                wal.clear().unwrap();

                assert_eq!(wal.generation, None);
            }

            #[test]
            fn clear_large_log_should_truncate_it() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &vec![1; TRUNCATE_SIZE as usize]).unwrap();

                wal.clear().unwrap();

                assert!(records(&wal).is_empty());
                assert_eq!(wal_len(&platform), 0);
                insert(&mut wal, 2, &[3]).unwrap();
                kill(&platform);
                assert_eq!(records(&open(&platform)), vec![(2, vec![3])]);
            }

            #[test]
            fn clear_legacy_log_should_truncate_it() {
                let platform = test_platform();
                platform
                    .fs()
                    .write(WAL, legacy_record(24, &[1, 2]))
                    .unwrap();
                let mut wal = open(&platform);

                wal.clear().unwrap();

                assert!(wal.is_empty());
                assert_eq!(wal_len(&platform), 0);
                insert(&mut wal, 2, &[3]).unwrap();
                kill(&platform);
                assert_eq!(
                    wal_bytes(&platform),
                    [
                        header(FIRST_GENERATION).to_vec(),
                        record_at(16, 2, &[3], FIRST_GENERATION)
                    ]
                    .concat()
                );
            }

            #[test]
            fn clear_failure_should_keep_records() {
                for (op, size) in [(FsOp::Write, 2), (FsOp::SetLen, TRUNCATE_SIZE)] {
                    let platform = test_platform();
                    let mut wal = open(&platform);
                    let value = vec![1; size as usize];
                    insert(&mut wal, 1, &value).unwrap();
                    platform.fs().set_io_error(op, WAL, ErrorKind::Other);

                    assert!(wal.clear().is_err(), "op: {op:?}");

                    assert_eq!(records(&wal), vec![(1, value)], "op: {op:?}");
                }
            }
        }

        mod discard_staged {
            use super::*;

            #[test]
            fn discard_staged_should_drop_staged_records() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1]).unwrap();
                wal.stage(2, &[2; 1]);

                wal.discard_staged();

                wal.write_staged().unwrap();
                assert_eq!(records(&wal), vec![(1, vec![1])]);
            }
        }

        mod sync {
            use super::*;

            #[test]
            fn sync_should_make_records_survive_crash() {
                // GIVEN a synced record followed by one not synced
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();
                wal.sync().unwrap();
                insert(&mut wal, 2, &[3]).unwrap();

                // WHEN the system crashes
                platform.fs().crash();

                // THEN only the synced record is in the reopened log
                assert_eq!(records(&open(&platform)), vec![(1, vec![1, 2])]);
            }

            #[test]
            fn sync_failure_should_fail() {
                let platform = test_platform();
                let mut wal = open(&platform);
                platform
                    .fs()
                    .set_io_error(FsOp::SyncData, WAL, ErrorKind::Other);

                assert!(wal.sync().is_err());
            }
        }

        mod is_empty {
            use super::*;

            #[test]
            fn is_empty_should_be_true_only_without_records() {
                let platform = test_platform();
                let mut wal = open(&platform);
                let new = wal.is_empty();

                insert(&mut wal, 1, &[]).unwrap();

                assert!(new);
                assert!(!wal.is_empty());
            }
        }

        mod new {
            use super::*;

            #[test]
            fn new_should_create_empty_log_file() {
                let platform = test_platform();

                let wal = open(&platform);

                assert!(wal.is_empty());
                kill(&platform);
                assert!(wal_bytes(&platform).is_empty());
            }

            #[test]
            fn new_should_load_valid_records() {
                let platform = test_platform();
                let log = [
                    header(7).to_vec(),
                    record_at(16, 1, &[1, 2], 7),
                    record_at(50, 2, &[], 7),
                ];
                platform.fs().write(WAL, log.concat()).unwrap();

                let wal = open(&platform);

                assert_eq!(records(&wal), vec![(1, vec![1, 2]), (2, vec![])]);
                assert_eq!(wal.generation, Some(7));
            }

            #[test]
            fn new_should_ignore_torn_and_stale_records() {
                let past_end = [2_u64.to_le_bytes(), 100_u64.to_le_bytes()].concat();
                let no_trailer = record_at(50, 2, &[3], 7)[..17].to_vec();

                for invalid in [
                    vec![1; 4],
                    vec![1; 12],
                    past_end,
                    no_trailer,
                    record_at(50, 2, &[3], 6),
                    record_at(60, 2, &[3], 7),
                ] {
                    let platform = test_platform();
                    let log = [
                        header(7).to_vec(),
                        record_at(16, 1, &[1, 2], 7),
                        invalid.clone(),
                    ];
                    platform.fs().write(WAL, log.concat()).unwrap();

                    let mut wal = open(&platform);
                    insert(&mut wal, 3, &[4]).unwrap();

                    kill(&platform);
                    assert_eq!(
                        records(&open(&platform)),
                        vec![(1, vec![1, 2]), (3, vec![4])],
                        "invalid: {invalid:?}"
                    );
                }
            }

            #[test]
            fn new_legacy_log_should_load_its_records() {
                for legacy in [
                    vec![legacy_record(24, &[1, 2]), legacy_record(40, &[])],
                    vec![legacy_record(0, &[]), legacy_record(24, &[3])],
                    vec![legacy_record(16, &[1; 8]), legacy_record(0, &[])],
                ] {
                    let platform = test_platform();
                    let expected: Vec<(u64, Vec<u8>)> = legacy
                        .iter()
                        .map(|record| {
                            let (pos, record) = record.split_at(8);
                            (
                                u64::from_le_bytes(pos.try_into().unwrap()),
                                record[8..].to_vec(),
                            )
                        })
                        .collect();
                    platform.fs().write(WAL, legacy.concat()).unwrap();

                    let wal = open(&platform);

                    assert_eq!(records(&wal), expected, "legacy: {legacy:?}");
                    kill(&platform);
                    assert_eq!(wal_bytes(&platform), legacy.concat(), "legacy: {legacy:?}");
                }
            }

            #[test]
            fn new_legacy_log_should_ignore_torn_record() {
                let platform = test_platform();
                let legacy = [legacy_record(24, &[1, 2]), legacy_record(40, &[3; 10])].concat();
                platform
                    .fs()
                    .write(WAL, &legacy[..legacy.len() - 1])
                    .unwrap();

                let wal = open(&platform);

                assert_eq!(records(&wal), vec![(24, vec![1, 2])]);
            }

            #[test]
            fn new_legacy_log_without_records_should_be_empty() {
                for legacy in [vec![1; 4], vec![1; 40]] {
                    let platform = test_platform();
                    platform.fs().write(WAL, &legacy).unwrap();

                    let wal = open(&platform);

                    assert!(wal.is_empty(), "legacy: {legacy:?}");
                    drop(wal);
                    assert_eq!(wal_len(&platform), 0, "legacy: {legacy:?}");
                }
            }

            #[test]
            fn new_failure_should_fail() {
                for op in [FsOp::Open, FsOp::Metadata, FsOp::Read] {
                    let platform = test_platform();
                    platform.fs().write(WAL, [1, 2, 3]).unwrap();
                    platform.fs().set_io_error(op, WAL, ErrorKind::Other);

                    assert!(WriteAheadLog::new(&platform, NAME).is_err(), "op: {op:?}");
                }
            }
        }

        mod rollback {
            use super::*;

            #[test]
            fn rollback_should_undo_records_from_newest() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 2, &[1]).unwrap();
                insert(&mut wal, 1, &[2, 3]).unwrap();

                let undone = undone(&wal);

                assert_eq!(undone, Ok(vec![(1, vec![2, 3]), (2, vec![1])]));
            }

            #[test]
            fn rollback_undo_failure_should_stop_and_fail() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 2, &[1]).unwrap();
                insert(&mut wal, 1, &[2, 3]).unwrap();
                let mut undone = Vec::new();

                let result = wal.rollback(|pos, _| {
                    undone.push(pos);
                    Err(DbError::storage(
                        crate::DbErrorType::NotFound,
                        "undo failed",
                    ))
                });

                assert_eq!(
                    result,
                    Err(DbError::storage(
                        crate::DbErrorType::NotFound,
                        "undo failed"
                    ))
                );
                assert_eq!(undone, vec![1]);
            }

            #[test]
            fn rollback_failure_should_fail() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::Read, WAL, ErrorKind::Other);

                assert!(undone(&wal).is_err());
            }
        }

        mod stage {
            use super::*;

            #[test]
            fn stage_should_write_nothing_until_write_staged() {
                let platform = test_platform();
                let mut wal = open(&platform);

                wal.stage(1, &[1; 2]);
                wal.stage(5, &[]);

                assert_eq!(wal_len(&platform), 0);
                assert!(wal.is_empty());
            }
        }

        mod stage_with {
            use super::*;

            #[test]
            fn stage_with_should_stage_value_written_into_buffer() {
                let platform = test_platform();
                let mut wal = open(&platform);

                let staged = wal
                    .stage_with(1, 3, &|buffer| {
                        buffer.copy_from_slice(&[1, 2, 3]);
                        Ok(())
                    })
                    .unwrap()
                    .to_vec();

                assert_eq!(staged, vec![1, 2, 3]);
                assert_eq!(wal_len(&platform), 0);
                wal.write_staged().unwrap();
                assert_eq!(records(&wal), vec![(1, vec![1, 2, 3])]);
            }

            #[test]
            fn stage_with_value_failure_should_fail_without_staging() {
                let platform = test_platform();
                let mut wal = open(&platform);

                let result = wal.stage_with(1, 3, &|_| {
                    Err(DbError::storage(
                        crate::DbErrorType::NotFound,
                        "read failed",
                    ))
                });

                assert_eq!(
                    result,
                    Err(DbError::storage(
                        crate::DbErrorType::NotFound,
                        "read failed"
                    ))
                );
                assert_eq!(wal.staged_len(), 0);
            }
        }

        mod staged_len {
            use super::*;

            #[test]
            fn staged_len_should_count_bytes_of_staged_records_and_header() {
                let platform = test_platform();
                let mut wal = open(&platform);

                wal.stage(1, &[1, 2]);

                assert_eq!(wal.staged_len(), 16 + 34);
                wal.write_staged().unwrap();
                assert_eq!(wal.staged_len(), 0);
            }
        }

        mod truncate {
            use super::*;

            #[test]
            fn truncate_should_discard_staged_records() {
                let platform = test_platform();
                let mut wal = open(&platform);
                wal.stage(1, &[1; 1]);

                wal.truncate().unwrap();

                wal.write_staged().unwrap();
                assert!(wal.is_empty());
                assert_eq!(wal_len(&platform), 0);
            }

            #[test]
            fn truncate_should_empty_log_file() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();

                wal.truncate().unwrap();

                assert!(wal.is_empty());
                assert_eq!(wal_len(&platform), 0);
                insert(&mut wal, 2, &[3]).unwrap();
                kill(&platform);
                assert_eq!(records(&open(&platform)), vec![(2, vec![3])]);
            }

            #[test]
            fn truncate_failure_should_keep_records() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();
                platform
                    .fs()
                    .set_io_error(FsOp::SetLen, WAL, ErrorKind::Other);

                assert!(wal.truncate().is_err());

                assert_eq!(records(&wal), vec![(1, vec![1, 2])]);
            }
        }

        mod write_staged {
            use super::*;

            #[test]
            fn write_staged_into_new_log_should_write_header_and_records_at_once() {
                let platform = test_platform();
                let mut wal = open(&platform);
                wal.stage(1, &[1; 2]);
                wal.stage(5, &[]);

                wal.write_staged().unwrap();

                kill(&platform);
                assert_eq!(
                    wal_bytes(&platform),
                    [
                        header(FIRST_GENERATION).to_vec(),
                        record_at(16, 1, &[1, 1], FIRST_GENERATION),
                        record_at(16 + 34, 5, &[], FIRST_GENERATION)
                    ]
                    .concat()
                );
            }

            #[test]
            fn write_staged_should_append_records_after_inserted_ones() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1]).unwrap();
                wal.stage(2, &[2; 1]);
                wal.stage(3, &[3; 1]);

                wal.write_staged().unwrap();
                insert(&mut wal, 4, &[4]).unwrap();

                kill(&platform);
                assert_eq!(
                    records(&open(&platform)),
                    vec![(1, vec![1]), (2, vec![2]), (3, vec![3]), (4, vec![4])]
                );
            }

            #[test]
            fn write_staged_after_clear_should_overwrite_invalidated_records() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 1, 1, 1]).unwrap();
                wal.clear().unwrap();
                wal.stage(2, &[2; 1]);

                wal.write_staged().unwrap();

                kill(&platform);
                assert_eq!(records(&open(&platform)), vec![(2, vec![2])]);
            }

            #[test]
            fn write_staged_empty_should_do_nothing() {
                let platform = test_platform();
                let mut wal = open(&platform);

                wal.write_staged().unwrap();

                assert_eq!(wal_len(&platform), 0);
            }

            #[test]
            fn write_staged_failure_should_keep_records_staged() {
                let platform = test_platform();
                let mut wal = open(&platform);
                wal.stage(1, &[1; 1]);
                platform
                    .fs()
                    .set_io_error(FsOp::Write, WAL, ErrorKind::StorageFull);

                assert!(wal.write_staged().is_err());

                platform.fs().clear_faults();
                wal.write_staged().unwrap();
                assert_eq!(records(&wal), vec![(1, vec![1])]);
            }

            #[test]
            fn write_staged_should_append_records_in_order() {
                let platform = test_platform();
                let mut wal = open(&platform);

                insert(&mut wal, 1, &[1, 2]).unwrap();
                insert(&mut wal, 5, &[]).unwrap();

                assert_eq!(records(&wal), vec![(1, vec![1, 2]), (5, vec![])]);
                assert_eq!(wal_len(&platform), 16 + 34 + 32);
            }

            #[test]
            fn write_staged_into_new_log_should_have_no_records_for_version_without_header() {
                // GIVEN a log not cleared before a crash
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();
                kill(&platform);

                // WHEN a version without the header reads its records
                let bytes = wal_bytes(&platform);
                let records = Records::new(&bytes, 0, None).count();

                // THEN there are none as the generation is a size past the end
                assert_eq!(records, 0);
            }

            #[test]
            fn write_staged_into_new_log_after_torn_write_should_ignore_its_remains() {
                // GIVEN a first write that wrote only a part of its header and record
                let platform = test_platform();
                let mut wal = open(&platform);
                tear_write(&platform, &mut wal);

                // WHEN a shorter record is written
                insert(&mut wal, 2, &[3]).unwrap();

                // THEN the reopened log holds just that record
                kill(&platform);
                assert_eq!(records(&open(&platform)), vec![(2, vec![3])]);
            }

            #[test]
            fn write_staged_after_torn_write_should_ignore_its_remains() {
                // GIVEN a write that wrote only a part of its record
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();
                tear_write(&platform, &mut wal);

                // WHEN a shorter record is written in its place
                insert(&mut wal, 3, &[4]).unwrap();

                // THEN the remains of the torn one past it are not a record
                kill(&platform);
                assert_eq!(
                    records(&open(&platform)),
                    vec![(1, vec![1, 2]), (3, vec![4])]
                );
            }

            #[test]
            fn write_staged_into_legacy_log_should_append_record_without_trailer() {
                let platform = test_platform();
                let legacy = legacy_record(24, &[1, 2]);
                platform.fs().write(WAL, &legacy).unwrap();
                let mut wal = open(&platform);

                insert(&mut wal, 2, &[3]).unwrap();

                kill(&platform);
                assert_eq!(
                    wal_bytes(&platform),
                    [legacy, legacy_record(2, &[3])].concat()
                );
                assert_eq!(
                    records(&open(&platform)),
                    vec![(24, vec![1, 2]), (2, vec![3])]
                );
            }
        }

        mod drop {
            use super::*;

            #[test]
            fn drop_empty_log_should_truncate_it() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();
                wal.clear().unwrap();

                drop(wal);

                assert_eq!(wal_len(&platform), 0);
            }

            #[test]
            fn drop_new_log_should_keep_it_empty() {
                let platform = test_platform();
                let wal = open(&platform);

                drop(wal);

                assert_eq!(wal_len(&platform), 0);
            }

            #[test]
            fn drop_should_keep_records() {
                let platform = test_platform();
                let mut wal = open(&platform);
                insert(&mut wal, 1, &[1, 2]).unwrap();

                drop(wal);

                assert_eq!(records(&open(&platform)), vec![(1, vec![1, 2])]);
            }
        }
    }

    mod records {
        use super::*;

        mod iterator {
            use super::*;

            mod next {
                use super::*;

                #[test]
                fn next_should_return_records_up_to_invalid_one() {
                    let valid = [record_at(16, 1, &[1, 2], 3), record_at(50, 2, &[], 3)].concat();
                    let past_end = [2_u64.to_le_bytes(), 9_u64.to_le_bytes()].concat();
                    let no_trailer = record_at(82, 3, &[3], 3)[..20].to_vec();
                    let no_end = record_at(82, 3, &[3], 3)[..25].to_vec();

                    for invalid in [
                        vec![],
                        vec![1; 7],
                        vec![1; 15],
                        past_end,
                        no_trailer,
                        no_end,
                        record_at(82, 3, &[3], 4),
                        record_at(90, 3, &[3], 3),
                    ] {
                        let bytes = [valid.clone(), invalid.clone()].concat();

                        let records: Vec<(u64, &[u8])> =
                            Records::new(&bytes, 16, Some(3)).collect();

                        assert_eq!(
                            records,
                            vec![(1, &[1_u8, 2][..]), (2, &[][..])],
                            "invalid: {invalid:?}"
                        );
                    }
                }

                #[test]
                fn next_without_generation_should_read_legacy_records() {
                    let bytes = [legacy_record(24, &[1, 2]), legacy_record(2, &[3])].concat();

                    let records: Vec<(u64, &[u8])> = Records::new(&bytes, 0, None).collect();

                    assert_eq!(records, vec![(24, &[1_u8, 2][..]), (2, &[3_u8][..])]);
                }
            }
        }
    }

    mod generation {
        use super::*;

        #[test]
        fn generation_should_read_header() {
            assert_eq!(generation(&header(5)), Some(5));
        }

        #[test]
        fn generation_without_header_should_be_none() {
            for bytes in [
                vec![],
                VERSION.to_le_bytes().to_vec(),
                legacy_record(0, &[]),
                legacy_record(16, &[1; 8]),
            ] {
                assert_eq!(generation(&bytes), None, "bytes: {bytes:?}");
            }
        }
    }

    mod append_record {
        use super::*;

        #[test]
        fn append_record_should_append_pos_size_zeroed_value_and_trailer() {
            let mut bytes = vec![9];

            let value_start = append_record(&mut bytes, 100, 1, 2, Some(3));

            assert_eq!(value_start, 17);
            assert_eq!(
                bytes,
                [
                    &[9][..],
                    &1_u64.to_le_bytes(),
                    &2_u64.to_le_bytes(),
                    &[0, 0],
                    &3_u64.to_le_bytes(),
                    &135_u64.to_le_bytes()
                ]
                .concat()
            );
        }

        #[test]
        fn append_record_without_generation_should_have_no_trailer() {
            let mut bytes = Vec::new();

            append_record(&mut bytes, 0, 1, 0, None);

            assert_eq!(bytes, legacy_record(1, &[]));
        }
    }

    mod wal_filename {
        use super::*;

        #[test]
        fn wal_filename_should_prefix_file_name_with_dot() {
            for (filename, expected) in [
                ("file", ".file"),
                ("/file", "/.file"),
                ("/some/path/file", "/some/path/.file"),
                ("\\some\\path\\file", "\\some\\path\\.file"),
            ] {
                assert_eq!(wal_filename(filename), expected, "filename: {filename}");
            }
        }
    }
}
