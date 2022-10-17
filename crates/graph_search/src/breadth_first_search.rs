use crate::search_index::SearchIndex;
use crate::search_iterator::SearchIterator;
use std::mem::swap;
use std::vec::IntoIter;

pub(crate) struct BreadthFirstSearch {
    stack_iterator: IntoIter<SearchIndex>,
}

impl BreadthFirstSearch {
    fn take_stack(stack: &mut Vec<SearchIndex>) -> Vec<SearchIndex> {
        let mut res = Vec::<SearchIndex>::new();
        swap(&mut res, stack);

        res
    }
}

impl SearchIterator for BreadthFirstSearch {
    fn new(stack: &mut Vec<SearchIndex>) -> Self {
        Self {
            stack_iterator: Self::take_stack(stack).into_iter(),
        }
    }

    fn next(&mut self) -> Option<SearchIndex> {
        self.stack_iterator.next()
    }
}
