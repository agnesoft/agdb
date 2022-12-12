use super::query_values::QueryValues;

pub struct InsertNodeQuery {
    pub count: u64,
    pub values: QueryValues,
}
