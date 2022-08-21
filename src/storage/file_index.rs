#[derive(Default)]
pub(crate) struct FileIndex {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_index_can_be_default_constructed() {
        let _index = FileIndex::default();
    }
}
