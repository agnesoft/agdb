pub(crate) struct BadFile {
    pub(crate) read_results: Vec<std::io::Result<usize>>,
    pub(crate) read_exact_results: Vec<std::io::Result<()>>,
    pub(crate) seek_results: Vec<std::io::Result<u64>>,
    pub(crate) write_results: Vec<std::io::Result<usize>>,
    pub(crate) write_all_results: Vec<std::io::Result<()>>,
    pub(crate) flush_results: Vec<std::io::Result<()>>,
}

impl std::io::Read for BadFile {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        let result;
        match if self.read_results.len() > 1 {
            result = self.read_results.remove(0);
            &result
        } else {
            self.read_results.first().unwrap()
        } {
            Ok(v) => Ok(*v),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }

    fn read_exact(&mut self, _buf: &mut [u8]) -> std::io::Result<()> {
        let result;
        match if self.read_exact_results.len() > 1 {
            result = self.read_exact_results.remove(0);
            &result
        } else {
            self.read_exact_results.first().unwrap()
        } {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }
}

impl std::io::Seek for BadFile {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let result;
        match if self.seek_results.len() > 1 {
            result = self.seek_results.remove(0);
            &result
        } else {
            self.seek_results.first().unwrap()
        } {
            Ok(v) => Ok(*v),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }
}

impl std::io::Write for BadFile {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        let result;
        match if self.write_results.len() > 1 {
            result = self.write_results.remove(0);
            &result
        } else {
            self.write_results.first().unwrap()
        } {
            Ok(v) => Ok(*v),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }

    fn write_all(&mut self, mut _buf: &[u8]) -> std::io::Result<()> {
        let result;
        match if self.write_all_results.len() > 1 {
            result = self.write_all_results.remove(0);
            &result
        } else {
            self.write_all_results.first().unwrap()
        } {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let result;
        match if self.flush_results.len() > 1 {
            result = self.flush_results.remove(0);
            &result
        } else {
            self.flush_results.first().unwrap()
        } {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }
}

impl Default for BadFile {
    fn default() -> Self {
        Self {
            read_results: vec![Ok(0)],
            read_exact_results: vec![Ok(())],
            seek_results: vec![Ok(0)],
            write_results: vec![Ok(0)],
            write_all_results: vec![Ok(())],
            flush_results: vec![Ok(())],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::io::Seek;
    use std::io::Write;

    use super::*;

    #[test]
    fn default_constructed() {
        let _file = BadFile::default();
    }

    #[test]
    fn flush_err() {
        let mut file = BadFile {
            flush_results: vec![Err(std::io::Error::from(std::io::ErrorKind::Other))],
            ..Default::default()
        };

        assert!(file.flush().is_err());
    }

    #[test]
    fn flush_list() {
        let mut file = BadFile {
            flush_results: vec![
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Ok(()),
            ],
            ..Default::default()
        };

        assert!(file.flush().is_err());
        assert!(file.flush().is_err());
        assert!(file.flush().is_ok());
        assert!(file.flush().is_ok());
    }

    #[test]
    fn flush_ok() {
        let mut file = BadFile::default();

        assert!(file.flush().is_ok());
    }

    #[test]
    fn read_exact_err() {
        let mut file = BadFile {
            read_exact_results: vec![Err(std::io::Error::from(std::io::ErrorKind::Other))],
            ..Default::default()
        };

        let mut buf = vec![0_u8; 0];
        assert!(file.read_exact(&mut buf).is_err());
    }

    #[test]
    fn read_exact_list() {
        let mut file = BadFile {
            read_exact_results: vec![
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Ok(()),
            ],
            ..Default::default()
        };

        let mut buf = vec![0_u8; 0];
        assert!(file.read_exact(&mut buf).is_err());
        assert!(file.read_exact(&mut buf).is_err());
        assert!(file.read_exact(&mut buf).is_ok());
        assert!(file.read_exact(&mut buf).is_ok());
    }

    #[test]
    fn read_exact_ok() {
        let mut file = BadFile::default();

        let mut buf = vec![0_u8; 0];
        assert!(file.read_exact(&mut buf).is_ok());
    }

    #[test]
    fn read_err() {
        let mut file = BadFile {
            read_results: vec![Err(std::io::Error::from(std::io::ErrorKind::Other))],
            ..Default::default()
        };

        let mut buf = vec![0_u8; 0];
        assert!(file.read(&mut buf).is_err());
    }

    #[test]
    fn read_list() {
        let mut file = BadFile {
            read_results: vec![
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Ok(0),
            ],
            ..Default::default()
        };

        let mut buf = vec![0_u8; 0];
        assert!(file.read(&mut buf).is_err());
        assert!(file.read(&mut buf).is_err());
        assert!(file.read(&mut buf).is_ok());
        assert!(file.read(&mut buf).is_ok());
    }

    #[test]
    fn read_ok() {
        let mut file = BadFile::default();

        let mut buf = vec![0_u8; 0];
        assert!(file.read(&mut buf).is_ok());
    }

    #[test]
    fn seek_err() {
        let mut file = BadFile {
            seek_results: vec![Err(std::io::Error::from(std::io::ErrorKind::Other))],
            ..Default::default()
        };

        assert!(file.seek(std::io::SeekFrom::Current(0)).is_err());
    }

    #[test]
    fn seek_list() {
        let mut file = BadFile {
            seek_results: vec![
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Ok(0),
            ],
            ..Default::default()
        };

        assert!(file.seek(std::io::SeekFrom::Current(0)).is_err());
        assert!(file.seek(std::io::SeekFrom::Current(0)).is_err());
        assert!(file.seek(std::io::SeekFrom::Current(0)).is_ok());
        assert!(file.seek(std::io::SeekFrom::Current(0)).is_ok());
    }

    #[test]
    fn seek_ok() {
        let mut file = BadFile::default();

        assert!(file.seek(std::io::SeekFrom::Current(0)).is_ok());
    }

    #[test]
    fn write_all_err() {
        let mut file = BadFile {
            write_all_results: vec![Err(std::io::Error::from(std::io::ErrorKind::Other))],
            ..Default::default()
        };

        let buf = vec![0_u8; 0];
        assert!(file.write_all(&buf).is_err());
    }

    #[test]
    fn write_all_list() {
        let mut file = BadFile {
            write_all_results: vec![
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Ok(()),
            ],
            ..Default::default()
        };

        let buf = vec![0_u8; 0];
        assert!(file.write_all(&buf).is_err());
        assert!(file.write_all(&buf).is_err());
        assert!(file.write_all(&buf).is_ok());
        assert!(file.write_all(&buf).is_ok());
    }

    #[test]
    fn write_all_ok() {
        let mut file = BadFile::default();

        let buf = vec![0_u8; 0];
        assert!(file.write_all(&buf).is_ok());
    }

    #[test]
    fn write_err() {
        let mut file = BadFile {
            write_results: vec![Err(std::io::Error::from(std::io::ErrorKind::Other))],
            ..Default::default()
        };

        let buf = vec![0_u8; 0];
        assert!(file.write(&buf).is_err());
    }

    #[test]
    fn write_list() {
        let mut file = BadFile {
            write_results: vec![
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Err(std::io::Error::from(std::io::ErrorKind::Other)),
                Ok(0),
            ],
            ..Default::default()
        };

        let buf = vec![0_u8; 0];
        assert!(file.write(&buf).is_err());
        assert!(file.write(&buf).is_err());
        assert!(file.write(&buf).is_ok());
        assert!(file.write(&buf).is_ok());
    }

    #[test]
    fn write_ok() {
        let mut file = BadFile::default();

        let buf = vec![0_u8; 0];
        assert!(file.write(&buf).is_ok());
    }
}
