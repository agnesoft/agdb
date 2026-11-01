//! A store of notes kept as files in a directory.
//!
//! The store takes `&impl Fs`, so its tests run against a `TestFs`: an
//! in-memory filesystem that starts empty, needs no cleanup, and can
//! simulate I/O failures that a real disk cannot produce on demand, e.g. a
//! write failing in the middle of saving a note.
//!
//! ```bash
//! cargo run --example fs --features fs
//! ```

use std::path::Path;

use agplatform::Env;
use agplatform::FileType;
use agplatform::Fs;
use agplatform::Platform;

/// Saves `content` as the note `name` in `dir`, creating `dir` if needed.
/// The note is written to a temporary file renamed over the previous one,
/// so a failed save leaves the previous content intact.
fn save(fs: &impl Fs, dir: &Path, name: &str, content: &str) -> agplatform::Result<()> {
    fs.create_dir_all(dir)?;
    let tmp = dir.join(format!(".{name}.tmp"));
    fs.write(&tmp, content)?;
    fs.rename(&tmp, dir.join(format!("{name}.md")))
}

/// Returns the names and contents of the notes in `dir` sorted by name.
fn load_all(fs: &impl Fs, dir: &Path) -> agplatform::Result<Vec<(String, String)>> {
    let mut notes = Vec::new();

    for entry in fs.read_dir(dir)? {
        let entry = entry?;
        if entry.file_type() == FileType::File
            && let Some(name) = entry.file_name().to_str()
            && let Some(name) = name.strip_suffix(".md")
        {
            notes.push((name.to_string(), fs.read_to_string(entry.path())?));
        }
    }

    notes.sort();
    Ok(notes)
}

fn main() -> agplatform::Result<()> {
    let platform = agplatform::platform();
    let dir = platform.env().tmp_dir().join("agplatform_example_notes");

    save(platform.fs(), &dir, "hello", "Hello, notes!")?;
    for (name, content) in load_all(platform.fs(), &dir)? {
        println!("{name}: {content}");
    }

    platform.fs().remove_dir_all(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agplatform::TestFs;
    use agplatform::test_platform::test_fs::FsOp;
    use std::io::ErrorKind;

    mod save {
        use super::*;

        #[test]
        fn save_should_create_missing_dir() {
            let fs = TestFs::new();

            save(&fs, Path::new("notes/2026"), "todo", "write examples").unwrap();

            assert_eq!(
                fs.read_to_string("notes/2026/todo.md").unwrap(),
                "write examples"
            );
        }

        #[test]
        fn save_should_replace_previous_content() {
            let fs = TestFs::new();
            save(&fs, Path::new("notes"), "todo", "old").unwrap();

            save(&fs, Path::new("notes"), "todo", "new").unwrap();

            assert_eq!(fs.read_to_string("notes/todo.md").unwrap(), "new");
            assert_eq!(fs.read_dir("notes").unwrap().count(), 1);
        }

        #[test]
        fn save_failing_write_should_keep_previous_content() {
            // GIVEN a saved note and a disk failing all writes from now on
            let fs = TestFs::new();
            save(&fs, Path::new("notes"), "todo", "old").unwrap();
            fs.set_io_error(FsOp::Write, "", ErrorKind::StorageFull);

            // WHEN saving it again
            let result = save(&fs, Path::new("notes"), "todo", "new");

            // THEN the save fails and the note keeps its previous content
            assert_eq!(result.unwrap_err().kind(), agplatform::ErrorKind::Fs);
            fs.clear_faults();
            assert_eq!(fs.read_to_string("notes/todo.md").unwrap(), "old");
        }
    }

    mod load_all {
        use super::*;

        #[test]
        fn load_all_should_return_notes_sorted_by_name() {
            let fs = TestFs::new();
            save(&fs, Path::new("notes"), "b", "second").unwrap();
            save(&fs, Path::new("notes"), "a", "first").unwrap();

            let notes = load_all(&fs, Path::new("notes")).unwrap();

            assert_eq!(
                notes,
                [
                    ("a".to_string(), "first".to_string()),
                    ("b".to_string(), "second".to_string())
                ]
            );
        }

        #[test]
        fn load_all_should_skip_other_files_and_dirs() {
            let fs = TestFs::new();
            fs.create_dir_all("notes/archive.md").unwrap();
            fs.write("notes/image.png", "").unwrap();
            fs.write("notes/note.md", "text").unwrap();

            let notes = load_all(&fs, Path::new("notes")).unwrap();

            assert_eq!(notes, [("note".to_string(), "text".to_string())]);
        }

        #[test]
        fn load_all_missing_dir_should_fail() {
            let fs = TestFs::new();

            let error = load_all(&fs, Path::new("notes")).unwrap_err();

            assert_eq!(error.kind(), agplatform::ErrorKind::Fs);
        }
    }
}
