pub(crate) struct BadFile {
    pub(crate) read_result: std::io::Result<usize>,
    pub(crate) read_exact_result: std::io::Result<()>,
    pub(crate) seek_result: std::io::Result<u64>,
    pub(crate) write_result: std::io::Result<usize>,
    pub(crate) write_all_result: std::io::Result<()>,
    pub(crate) flush_result: std::io::Result<()>,
}

impl std::io::Read for BadFile {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        match &self.read_result {
            Ok(v) => Ok(*v),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }

    fn read_exact(&mut self, _buf: &mut [u8]) -> std::io::Result<()> {
        match &self.read_exact_result {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }
}

impl std::io::Seek for BadFile {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match &self.seek_result {
            Ok(v) => Ok(*v),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }
}

impl std::io::Write for BadFile {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        match &self.write_result {
            Ok(v) => Ok(*v),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }

    fn write_all(&mut self, mut _buf: &[u8]) -> std::io::Result<()> {
        match &self.write_all_result {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match &self.flush_result {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }
}

impl Default for BadFile {
    fn default() -> Self {
        Self {
            read_result: Ok(0),
            read_exact_result: Ok(()),
            seek_result: Ok(0),
            write_result: Ok(0),
            write_all_result: Ok(()),
            flush_result: Ok(()),
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
    fn flush_ok() {
        let mut file = BadFile::default();

        assert!(file.flush().is_ok());
    }

    #[test]
    fn flush_err() {
        let mut file = BadFile {
            flush_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            ..Default::default()
        };

        assert!(file.flush().is_err());
    }

    #[test]
    fn read_exact_ok() {
        let mut file = BadFile::default();

        let mut buf = vec![0_u8; 0];
        assert!(file.read_exact(&mut buf).is_ok());
    }

    #[test]
    fn read_exact_err() {
        let mut file = BadFile {
            read_exact_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            ..Default::default()
        };

        let mut buf = vec![0_u8; 0];
        assert!(file.read_exact(&mut buf).is_err());
    }

    #[test]
    fn read_ok() {
        let mut file = BadFile::default();

        let mut buf = vec![0_u8; 0];
        assert!(file.read(&mut buf).is_ok());
    }

    #[test]
    fn read_err() {
        let mut file = BadFile {
            read_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            ..Default::default()
        };

        let mut buf = vec![0_u8; 0];
        assert!(file.read(&mut buf).is_err());
    }

    #[test]
    fn seek_ok() {
        let mut file = BadFile::default();

        assert!(file.seek(std::io::SeekFrom::Current(0)).is_ok());
    }

    #[test]
    fn seek_err() {
        let mut file = BadFile {
            seek_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            ..Default::default()
        };

        assert!(file.seek(std::io::SeekFrom::Current(0)).is_err());
    }

    #[test]
    fn write_all_ok() {
        let mut file = BadFile::default();

        let buf = vec![0_u8; 0];
        assert!(file.write_all(&buf).is_ok());
    }

    #[test]
    fn write_all_err() {
        let mut file = BadFile {
            write_all_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            ..Default::default()
        };

        let buf = vec![0_u8; 0];
        assert!(file.write_all(&buf).is_err());
    }

    #[test]
    fn write_ok() {
        let mut file = BadFile::default();

        let buf = vec![0_u8; 0];
        assert!(file.write(&buf).is_ok());
    }

    #[test]
    fn write_err() {
        let mut file = BadFile {
            write_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            ..Default::default()
        };

        let buf = vec![0_u8; 0];
        assert!(file.write(&buf).is_err());
    }
}
