#[derive(Clone, Debug, Default, PartialEq)]
#[allow(dead_code)]
pub(crate) struct WriteAheadLogRecord {
    pub(super) position: u64,
    pub(super) bytes: Vec<u8>,
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
