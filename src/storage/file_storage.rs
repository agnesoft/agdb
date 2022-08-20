#[allow(dead_code)]
#[derive(Default)]
pub(crate) struct FileStorage {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_storage() {
        let _storage = FileStorage::default();
    }
}
