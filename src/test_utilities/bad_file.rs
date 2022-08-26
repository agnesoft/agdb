use std::io::{Read, Result, Seek, SeekFrom, Write};

struct BadFile {
    read_result: Result<usize>,
    seek_result: Result<u64>,
    write_result: Result<usize>,
    flush_result: Result<()>,
}

impl Read for BadFile {
    fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_result
    }
}

impl Seek for BadFile {
    fn seek(&mut self, _pos: SeekFrom) -> std::io::Result<u64> {
        self.seek_result
    }
}

impl Write for BadFile {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        self.write_result
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_result
    }
}
