#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DictionaryIndex(pub u64);

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn derived_from_clone() {
        let _ = DictionaryIndex(0).clone();
    }

    #[test]
    fn derived_from_debug() {
        let _ = format!("{:?}", DictionaryIndex(0));
    }

    #[test]
    fn derived_from_default() {
        let _ = DictionaryIndex::default();
    }

    #[test]
    fn derived_from_ord() {
        assert_eq!(DictionaryIndex(1).cmp(&DictionaryIndex(1)), Ordering::Equal);
    }

    #[test]
    fn derived_from_partial_ord() {
        let mut ids = vec![DictionaryIndex(3), DictionaryIndex(0), DictionaryIndex(1)];
        ids.sort();

        assert_eq!(
            ids,
            vec![DictionaryIndex(0), DictionaryIndex(1), DictionaryIndex(3)]
        );
    }
}
