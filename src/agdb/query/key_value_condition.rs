use crate::DbKey;

use super::comparison::Comparison;

pub struct KeyValueCondition {
    pub key: DbKey,
    pub comparison: Comparison,
}
