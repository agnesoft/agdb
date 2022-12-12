use crate::Query;

#[allow(dead_code)]
pub enum QueryIds {
    Ids(Vec<u64>),
    Query(Box<Query>),
}
