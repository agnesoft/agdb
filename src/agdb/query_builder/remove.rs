use super::remove_alias::RemoveAlias;
use super::remove_values::RemoveValues;
use crate::query::query_ids::QueryIds;
use crate::query::remove_aliases_query::RemoveAliasesQuery;
use crate::query::remove_values_query::RemoveValuesQuery;
use crate::query::select_query::SelectQuery;
use crate::DbKey;

pub struct Remove {}

impl Remove {
    pub fn alias(self, name: &str) -> RemoveAlias {
        RemoveAlias(RemoveAliasesQuery {
            aliases: vec![name.to_string()],
        })
    }

    pub fn aliases(self, names: &[String]) -> RemoveAlias {
        RemoveAlias(RemoveAliasesQuery {
            aliases: names.to_vec(),
        })
    }

    pub fn value(self, key: DbKey) -> RemoveValues {
        RemoveValues(RemoveValuesQuery(SelectQuery {
            keys: vec![key],
            ids: QueryIds::Id(0.into()),
        }))
    }

    pub fn values(self, keys: &[DbKey]) -> RemoveValues {
        RemoveValues(RemoveValuesQuery(SelectQuery {
            keys: keys.to_vec(),
            ids: QueryIds::Id(0.into()),
        }))
    }
}
