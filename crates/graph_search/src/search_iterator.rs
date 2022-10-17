use crate::search_index::SearchIndex;

pub(crate) trait SearchIterator {
    fn new(stack: &mut Vec<SearchIndex>) -> Self;
    fn next(&mut self) -> Option<SearchIndex>;
}
