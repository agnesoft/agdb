use crate::SearchControl;

pub trait SearchHandler {
    fn process(&mut self, data: &mut SearchData) -> SearchControl;
}
