use crate::db_error::DbError;

#[allow(dead_code)]
pub(crate) struct WriteAheadLog {
    file: std::fs::File,
}

impl TryFrom<String> for WriteAheadLog {
    type Error = DbError;

    fn try_from(filename: String) -> Result<Self, Self::Error> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&filename)?;

        Ok(WriteAheadLog { file })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::test_file::TestFile;

    #[test]
    fn filename_constructed() {
        let test_file = TestFile::from("./write_ahead_log-filename_constructed.agdb");
        WriteAheadLog::try_from(test_file.file_name().clone()).unwrap();
    }
}
