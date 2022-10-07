use std::path::Path;

pub struct TestFile {
    pub(crate) filename: String,
}

impl TestFile {
    pub fn file_name(&self) -> &String {
        &self.filename
    }

    #[track_caller]
    pub fn new() -> TestFile {
        let caller = std::panic::Location::caller();
        let file = format!(
            "./{}.{}.{}.testfile",
            Path::new(caller.file())
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            caller.line(),
            caller.column()
        );

        TestFile::from(file)
    }

    pub(crate) fn hidden_filename(filename: &String) -> String {
        let path = Path::new(filename);
        let name: String = path.file_name().unwrap().to_str().unwrap().to_string();
        let parent = path.parent().unwrap();

        parent
            .join(&Path::new(&(".".to_string() + &name)))
            .to_str()
            .unwrap()
            .to_string()
    }

    pub(crate) fn remove_file_if_exists(filename: &String) {
        if Path::new(filename).exists() {
            std::fs::remove_file(filename).unwrap();
        }
    }
}
