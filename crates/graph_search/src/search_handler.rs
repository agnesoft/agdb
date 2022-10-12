use crate::SearchControl;
use crate::SearchData;

pub trait SearchHandler {
    fn process(&self, data: &mut SearchData) -> SearchControl;
}
