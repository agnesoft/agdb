use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct FileIndex {
    positions: HashMap<i64, u64>,
    free_list: Vec<i64>,
}

#[allow(dead_code)]
impl FileIndex {
    pub(crate) fn get(&self, index: i64) -> Option<&u64> {
        self.positions.get(&index)
    }

    pub(crate) fn insert(&mut self, position: u64) -> i64 {
        let mut index = self.positions.len() as i64;

        if let Some(free_index) = self.free_list.pop() {
            index = free_index;
        }

        self.positions.insert(index, position);
        index
    }

    pub(crate) fn remove(&mut self, index: i64) {
        self.positions.remove(&index);
        self.free_list.push(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_index_can_be_default_constructed() {
        let _file_index = FileIndex::default();
    }

    #[test]
    fn insert_indexes() {
        let mut file_index = FileIndex::default();
        let pos1 = 32u64;
        let pos2 = 64u64;

        let index1 = file_index.insert(pos1);
        let index2 = file_index.insert(pos2);

        assert_eq!(file_index.get(index1), Some(&pos1));
        assert_eq!(file_index.get(index2), Some(&pos2));
    }

    #[test]
    fn remove_index() {
        let mut file_index = FileIndex::default();
        let pos1 = 32u64;
        let pos2 = 64u64;
        let index1 = file_index.insert(pos1);
        let index2 = file_index.insert(pos2);

        file_index.remove(index1);

        assert_eq!(file_index.get(index1), None);
        assert_eq!(file_index.get(index2), Some(&pos2));
    }

    #[test]
    fn reuse_index() {
        let mut file_index = FileIndex::default();
        let pos1 = 32u64;
        let pos2 = 64u64;
        let index1 = file_index.insert(pos1);
        let index2 = file_index.insert(pos2);

        file_index.remove(index1);
        file_index.remove(index2);
        let index3 = file_index.insert(pos1);
        let index4 = file_index.insert(pos2);

        assert_eq!(index3, index2);
        assert_eq!(index4, index1);
    }
}
