#[derive(Default)]
pub(crate) struct FileIndex {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_index_can_be_default_constructed() {
        let _file_index = FileIndex::default();
    }

    #[test]
    fn insert_indexes_and_positions() {
        let mut file_index = FileIndex::default();
        let index1 = 1_i64;
        let index2 = 2_i64;
        let pos1 = 32u64;
        let pos2 = 64u64;

        file_index.insert(index1, pos1);
        file_index.insert(index2, pos2);

        assert_eq!(file_index.get(index1), Some(&pos1));
        assert_eq!(file_index.get(index2), Some(&pos2));
    }
}
