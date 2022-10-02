use super::dictionary_data_memory::DictionaryDataMemory;
use super::dictionary_impl::DictionaryImpl;

pub(crate) type Dictionary = DictionaryImpl<DictionaryDataMemory>;

impl Dictionary {
    pub(crate) fn new() -> Dictionary {
        Dictionary {
            data: DictionaryDataMemory {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index() {}

    #[test]
    fn insert() {}

    #[test]
    fn remove() {}

    #[test]
    fn value() {}
}
