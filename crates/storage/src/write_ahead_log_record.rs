#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub struct WriteAheadLogRecord {
    pub position: u64,
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        let record = WriteAheadLogRecord::default();
        format!("{:?}", record);
    }
}
