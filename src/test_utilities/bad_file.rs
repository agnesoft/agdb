struct BadFile {
    read_result: std::io::Result<usize>,
    seek_result: std::io::Result<u64>,
    write_result: std::io::Result<usize>,
    flush_result: std::io::Result<()>,
}

impl std::io::Read for BadFile {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        match &self.read_result {
            Ok(v) => Ok(v.clone()),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }
}

impl std::io::Seek for BadFile {
    fn seek(&mut self, _pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match &self.seek_result {
            Ok(v) => Ok(v.clone()),
            Err(e) => Err(std::io::Error::from(e.kind())),
        }
    }
}

impl std::io::Write for BadFile {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        match &self.write_result {
            Ok(v) => Ok(v.clone()),
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

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::io::Seek;
    use std::io::Write;

    use super::*;

    #[test]
    fn flush_ok() {
        let mut file = BadFile {
            read_result: Ok(0),
            seek_result: Ok(0),
            write_result: Ok(0),
            flush_result: Ok(()),
        };

        assert!(file.flush().is_ok());
    }

    #[test]
    fn flush_err() {
        let mut file = BadFile {
            read_result: Ok(0),
            seek_result: Ok(0),
            write_result: Ok(0),
            flush_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
        };

        assert!(file.flush().is_err());
    }

    #[test]
    fn read_ok() {
        let mut file = BadFile {
            read_result: Ok(0),
            seek_result: Ok(0),
            write_result: Ok(0),
            flush_result: Ok(()),
        };

        let mut buf = vec![0_u8; 0];
        assert!(file.read(&mut buf).is_ok());
    }

    #[test]
    fn read_err() {
        let mut file = BadFile {
            read_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            seek_result: Ok(0),
            write_result: Ok(0),
            flush_result: Ok(()),
        };

        let mut buf = vec![0_u8; 0];
        assert!(file.read(&mut buf).is_err());
    }

    #[test]
    fn seek_ok() {
        let mut file = BadFile {
            read_result: Ok(0),
            seek_result: Ok(0),
            write_result: Ok(0),
            flush_result: Ok(()),
        };

        assert!(file.seek(std::io::SeekFrom::Current(0)).is_ok());
    }

    #[test]
    fn seek_err() {
        let mut file = BadFile {
            read_result: Ok(0),
            seek_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            write_result: Ok(0),
            flush_result: Ok(()),
        };

        assert!(file.seek(std::io::SeekFrom::Current(0)).is_err());
    }

    #[test]
    fn write_ok() {
        let mut file = BadFile {
            read_result: Ok(0),
            seek_result: Ok(0),
            write_result: Ok(0),
            flush_result: Ok(()),
        };

        let buf = vec![0_u8; 0];
        assert!(file.write(&buf).is_ok());
    }

    #[test]
    fn write_err() {
        let mut file = BadFile {
            read_result: Ok(0),
            seek_result: Ok(0),
            write_result: Err(std::io::Error::from(std::io::ErrorKind::Other)),
            flush_result: Ok(()),
        };

        let buf = vec![0_u8; 0];
        assert!(file.write(&buf).is_err());
    }
}
