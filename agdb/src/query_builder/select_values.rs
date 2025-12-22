use crate::Comparison;
use crate::DbValue;
use crate::KeyValueComparison;
use crate::QueryCondition;
use crate::QueryConditionData;
use crate::QueryConditionLogic;
use crate::QueryConditionModifier;
use crate::QueryIds;
use crate::SearchQuery;
use crate::SelectValuesQuery;
use crate::query_builder::search::Search;
use crate::query_builder::where_::DB_ELEMENT_ID_KEY;

/// Select values builder.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct SelectValues {
    pub query: SelectValuesQuery,
    pub element_id: Option<DbValue>,
    pub limit: u64,
}

/// Final builder that lets you create
/// an actual query object.
#[cfg_attr(feature = "api", derive(agdb::TypeDef))]
pub struct SelectValuesIds(pub SelectValuesQuery);

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectValues {
    /// An id or list of ids or search query to select values of.
    /// All ids specified must exist in the database.
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectValuesIds {
        self.query.ids = ids.into();

        SelectValuesIds(self.query)
    }

    /// Select using the result of a search query as ids.
    /// Equivalent to `ids(QueryBuilder::search()/* ... */)`.
    pub fn search(mut self) -> Search<SelectValuesQuery> {
        let mut search = SearchQuery::new();
        search.limit = self.limit;
        if let Some(element_id) = self.element_id {
            search.conditions.push(QueryCondition {
                logic: QueryConditionLogic::And,
                modifier: QueryConditionModifier::None,
                data: QueryConditionData::KeyValue(KeyValueComparison {
                    key: DB_ELEMENT_ID_KEY.into(),
                    value: Comparison::Equal(element_id),
                }),
            });
        }
        self.query.ids = QueryIds::Search(search);
        Search(self.query)
    }
}

#[cfg_attr(feature = "api", agdb::impl_def())]
impl SelectValuesIds {
    /// Returns the built `SelectValuesQuery` object.
    pub fn query(self) -> SelectValuesQuery {
        self.0
    }
}
