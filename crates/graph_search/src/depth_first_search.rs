use crate::search_index::SearchIndex;
use crate::search_iterator::SearchIterator;

pub(crate) struct DepthFirstSearch {
    index: Option<SearchIndex>,
}

impl SearchIterator for DepthFirstSearch {
    fn new(stack: &mut Vec<SearchIndex>) -> Self {
        Self { index: stack.pop() }
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.index.take()
    }
}
