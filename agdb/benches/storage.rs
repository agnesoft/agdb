use agbench::Bench;
use agdb::FileStorage;
use agdb::FileStorageMemoryMapped;
use agdb::MemoryStorage;
use agdb::StorageData;
use agdb::SyncMode;
use agplatform::TestDir;
use std::process::ExitCode;

const LEN: u64 = 1 << 20;
const SMALL: usize = 64;
const LARGE: usize = 64 << 10;

fn main() -> ExitCode {
    agbench::main(&[
        new,
        read_small,
        read_large,
        write_small,
        write_large,
        append,
        append_truncate,
        sync_mode,
    ])
}

/// Each storage kind holding `LEN` committed bytes in its own file of a
/// temporary directory.
struct Storages {
    memory: MemoryStorage,
    mapped: FileStorageMemoryMapped,
    file: FileStorage,
    _dir: TestDir,
}

fn path(dir: &TestDir, name: &str) -> String {
    dir.path()
        .join(name)
        .to_str()
        .expect("temp dir should be valid UTF-8")
        .to_string()
}

fn filled<S: StorageData>(name: &str) -> S {
    let mut storage = S::new(name).expect("storage should be created");
    storage
        .write(0, &vec![1_u8; LEN as usize])
        .expect("storage should be writable");
    storage.flush().expect("storage should be flushed");
    storage
}

fn storages() -> Storages {
    let dir = TestDir::temp();

    Storages {
        memory: filled(&path(&dir, "memory.agdb")),
        mapped: filled(&path(&dir, "mapped.agdb")),
        file: filled(&path(&dir, "file.agdb")),
        _dir: dir,
    }
}

/// Positions `step` bytes apart, wrapping before `LEN - len`, so that
/// consecutive iterations do not touch the same bytes.
fn positions(len: usize) -> impl FnMut() -> u64 {
    let mut pos = 0;

    move || {
        pos = (pos + 4096) % (LEN - len as u64);
        pos
    }
}

fn new(bench: &mut Bench) {
    bench
        .benchmark("storage/new_1mib", || {
            let dir = TestDir::temp();
            let name = path(&dir, "db.agdb");
            drop(filled::<FileStorage>(&name));
            (dir, name)
        })
        .target("mapped", |(_, name)| {
            FileStorageMemoryMapped::new(name).unwrap().len()
        })
        .target("memory", |(_, name)| {
            MemoryStorage::new(name).unwrap().len()
        })
        .target("file", |(_, name)| FileStorage::new(name).unwrap().len())
        .run();
}

fn read(bench: &mut Bench, name: &str, len: usize) {
    let (mut memory, mut mapped, mut file) = (positions(len), positions(len), positions(len));

    bench
        .benchmark(name, storages)
        .target("mapped", move |s| {
            s.mapped.read(mapped(), len as u64).unwrap()[0]
        })
        .target("memory", move |s| {
            s.memory.read(memory(), len as u64).unwrap()[0]
        })
        .target("file", move |s| s.file.read(file(), len as u64).unwrap()[0])
        .run();
}

fn read_small(bench: &mut Bench) {
    read(bench, "storage/read_64b", SMALL);
}

fn read_large(bench: &mut Bench) {
    read(bench, "storage/read_64kib", LARGE);
}

fn commit<S: StorageData>(storage: &mut S, pos: u64, bytes: &[u8]) {
    storage.write(pos, bytes).unwrap();
    storage.flush().unwrap();
}

fn write(bench: &mut Bench, name: &str, len: usize) {
    let (mut memory, mut mapped, mut file) = (positions(len), positions(len), positions(len));
    let bytes = vec![2_u8; len];

    bench
        .benchmark(name, storages)
        .target("mapped", |s| commit(&mut s.mapped, mapped(), &bytes))
        .target("memory", |s| commit(&mut s.memory, memory(), &bytes))
        .target("file", |s| commit(&mut s.file, file(), &bytes))
        .run();
}

fn write_small(bench: &mut Bench) {
    write(bench, "storage/write_64b_commit", SMALL);
}

fn write_large(bench: &mut Bench) {
    write(bench, "storage/write_64kib_commit", LARGE);
}

fn append_commit<S: StorageData>(storage: &mut S, bytes: &[u8]) {
    // WHY: the fastest target runs the most iterations, so unbounded it
    // would also measure growing a far larger buffer than the others.
    if storage.len() >= 2 * LEN {
        storage.resize(LEN).unwrap();
    }

    let len = storage.len();
    commit(storage, len, bytes);
}

fn append(bench: &mut Bench) {
    let bytes = [3_u8; SMALL];

    bench
        .benchmark("storage/append_64b_commit", storages)
        .target("mapped", |s| append_commit(&mut s.mapped, &bytes))
        .target("memory", |s| append_commit(&mut s.memory, &bytes))
        .target("file", |s| append_commit(&mut s.file, &bytes))
        .run();
}

fn append_truncate_commit<S: StorageData>(storage: &mut S, bytes: &[u8]) {
    storage.write(LEN, bytes).unwrap();
    storage.resize(LEN).unwrap();
    storage.flush().unwrap();
}

fn append_truncate(bench: &mut Bench) {
    let bytes = [3_u8; SMALL];

    bench
        .benchmark("storage/append_truncate_64b_commit", storages)
        .target("mapped", |s| append_truncate_commit(&mut s.mapped, &bytes))
        .target("memory", |s| append_truncate_commit(&mut s.memory, &bytes))
        .target("file", |s| append_truncate_commit(&mut s.file, &bytes))
        .run();
}

fn sync_mode(bench: &mut Bench) {
    let (mut none, mut commit_mode) = (positions(SMALL), positions(SMALL));
    let bytes = [2_u8; SMALL];

    bench
        .benchmark("storage/sync_mode_write_64b_commit", || {
            let dir = TestDir::temp();
            let none: FileStorage = filled(&path(&dir, "none.agdb"));
            let mut commit: FileStorage = filled(&path(&dir, "commit.agdb"));
            commit.set_sync_mode(SyncMode::Commit);
            (none, commit, dir)
        })
        .target("commit", |(_, s, _)| commit(s, commit_mode(), &bytes))
        .target("none", |(s, _, _)| commit(s, none(), &bytes))
        .run();
}
