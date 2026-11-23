use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryResult;
use crate::SearchQueryAlgorithm;
use crate::StorageData;
use crate::collections::key_values::KeyValues;
use crate::collections::keys::ALIAS;
use crate::collections::keys::KeyId;
use crate::db::db_search_handlers::DefaultHandler;
use crate::db::db_search_handlers::LimitHandler;
use crate::db::db_search_handlers::LimitOffsetHandler;
use crate::db::db_search_handlers::OffsetHandler;
use crate::db::db_search_handlers::PathHandler;
use crate::db::rebuild::Element;
use crate::db::rebuild::Source;
use crate::graph_search::GraphSearch;
use crate::graph_search::SearchControl;
use crate::query::Query;
use crate::query::query_condition::QueryCondition;
use crate::query::query_condition::QueryConditionData;
use crate::query::query_condition::QueryConditionLogic;
use crate::query::query_condition::QueryConditionModifier;
use crate::query::query_id::QueryId;
use crate::storage::StorageSlice;
use crate::storage::pages::Reader;
use crate::storage::pages::View;
use crate::storage::pages::Writer;
use crate::storage::root::Meta;
use std::borrow::Cow;

/// The pages a [`Transaction`] reads: the committed ones or those of the
/// write transaction it runs in.
pub(crate) enum ReadView<'a, S: StorageData> {
    Committed(Reader<'a, S>),
    Writing(&'a Writer<'a, S>),
}

impl<S: StorageData> View for ReadView<'_, S> {
    fn len(&self) -> u64 {
        match self {
            Self::Committed(reader) => reader.len(),
            Self::Writing(writer) => writer.len(),
        }
    }

    fn read(&self, pos: u64, len: u64) -> Result<StorageSlice<'_>, DbError> {
        match self {
            Self::Committed(reader) => reader.read(pos, len),
            Self::Writing(writer) => writer.read(pos, len),
        }
    }
}

/// The `Transaction` is a read-only view of the database. It allows
/// running queries via [`exec()`](#method.exec).
pub struct Transaction<'a, Store: StorageData> {
    view: ReadView<'a, Store>,
    meta: Cow<'a, Meta>,
    /// Why the database cannot be read, returned by every query.
    error: Option<DbError>,
}

impl<'a, Store: StorageData> Transaction<'a, Store> {
    /// Executes immutable queries:
    ///
    /// - Select elements
    /// - Select values
    /// - Select keys
    /// - Select key count
    /// - Select edge count
    /// - Select aliases
    /// - Select all aliases
    /// - Select indexes
    /// - Select node count
    /// - Search
    pub fn exec<T: Query>(&self, query: T) -> Result<QueryResult, DbError> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }

        query.process(self)
    }

    pub(crate) fn new(view: ReadView<'a, Store>, meta: Cow<'a, Meta>) -> Self {
        Self {
            view,
            meta,
            error: None,
        }
    }

    /// A transaction whose queries fail with the `error`.
    pub(crate) fn failed(reader: Reader<'a, Store>, error: DbError) -> Self {
        Self {
            view: ReadView::Committed(reader),
            meta: Cow::Owned(Meta::default()),
            error: Some(error),
        }
    }

    fn key_values(&self) -> KeyValues<'_> {
        KeyValues(&self.meta.graph)
    }

    pub(crate) fn alias(&self, id: DbId) -> Result<String, DbError> {
        match self.find_alias(id)? {
            Some(alias) => Ok(alias),
            None => Err(DbError::db(
                DbErrorType::NotFound,
                format!("Id '{}' not found", id.0),
            )),
        }
    }

    pub(crate) fn find_alias(&self, id: DbId) -> Result<Option<String>, DbError> {
        match self.key_values().value(&self.view, id, ALIAS)? {
            Some(alias) => Ok(Some(alias.string()?.clone())),
            None => Ok(None),
        }
    }

    pub(crate) fn aliases(&self) -> Result<Vec<(String, DbId)>, DbError> {
        let mut aliases = Vec::new();

        for (alias, ids) in self.meta.indexes.aliases().values(&self.view)? {
            aliases.push((alias.string()?.clone(), ids[0]));
        }

        Ok(aliases)
    }

    /// The node with the `alias`.
    pub(crate) fn alias_id(&self, alias: &str) -> Result<Option<DbId>, DbError> {
        Ok(self
            .meta
            .indexes
            .aliases()
            .search(&self.view, &DbValue::from(alias))?
            .first()
            .copied())
    }

    pub(crate) fn db_id(&self, query_id: &QueryId) -> Result<DbId, DbError> {
        match query_id {
            QueryId::Id(id) => {
                if self.exists(*id)? {
                    Ok(*id)
                } else {
                    Err(DbError::db(
                        DbErrorType::NotFound,
                        format!("Id '{}' not found", id.0),
                    ))
                }
            }
            QueryId::Alias(alias) => self.alias_id(alias)?.ok_or(DbError::db(
                DbErrorType::NotFound,
                format!("Alias '{alias}' not found"),
            )),
        }
    }

    pub(crate) fn exists(&self, id: DbId) -> Result<bool, DbError> {
        Ok(if id.0 < 0 {
            self.meta.graph.edge(&self.view, id)?.is_some()
        } else {
            self.meta.graph.node(&self.view, id)?.is_some()
        })
    }

    pub(crate) fn all_edge_count(&self) -> u64 {
        self.meta.graph.edge_count()
    }

    pub(crate) fn edge_count(&self, id: DbId, from: bool, to: bool) -> Result<u64, DbError> {
        let id = self.db_id(&QueryId::Id(id))?;

        Ok(match self.meta.graph.node(&self.view, id)? {
            Some(node) => u64::from(from) * node.from_count + u64::from(to) * node.to_count,
            None => 0,
        })
    }

    /// The origin of an edge, the first edge from a node.
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn from_id(&self, id: DbId) -> Result<DbId, DbError> {
        Ok(if id.0 < 0 {
            self.meta
                .graph
                .edge(&self.view, id)?
                .map_or(DbId(0), |edge| edge.from)
        } else {
            self.meta
                .graph
                .node(&self.view, id)?
                .map_or(DbId(0), |node| node.first_from)
        })
    }

    /// The destination of an edge, the first edge to a node.
    pub(crate) fn to_id(&self, id: DbId) -> Result<DbId, DbError> {
        Ok(if id.0 < 0 {
            self.meta
                .graph
                .edge(&self.view, id)?
                .map_or(DbId(0), |edge| edge.to)
        } else {
            self.meta
                .graph
                .node(&self.view, id)?
                .map_or(DbId(0), |node| node.first_to)
        })
    }

    pub(crate) fn indexes(&self) -> Vec<DbKeyValue> {
        self.meta
            .indexes
            .list()
            .iter()
            .map(|index| DbKeyValue {
                key: index.key.clone(),
                value: index.count.into(),
            })
            .collect()
    }

    pub(crate) fn keys(&self, id: DbId) -> Result<Vec<DbValue>, DbError> {
        self.key_values().keys(&self.view, &self.meta.keys, id)
    }

    pub(crate) fn key_count(&self, id: DbId) -> Result<u64, DbError> {
        self.key_values().key_count(&self.view, id)
    }

    /// The ids of the `keys` the database has, with the keys.
    pub(crate) fn key_ids<'k>(
        &self,
        keys: &'k [DbValue],
    ) -> Result<Vec<(KeyId, &'k DbValue)>, DbError> {
        let mut ids = Vec::new();

        for key in keys {
            if let Some(id) = self.meta.keys.id(&self.view, key)? {
                ids.push((id, key));
            }
        }

        Ok(ids)
    }

    pub(crate) fn node_count(&self) -> u64 {
        self.meta.graph.node_count()
    }

    pub(crate) fn search_index(
        &self,
        key: &DbValue,
        value: &DbValue,
    ) -> Result<Vec<DbId>, DbError> {
        self.meta
            .indexes
            .search(&self.view, key, value)?
            .ok_or(DbError::db(
                DbErrorType::NotFound,
                format!("Index '{key}' not found"),
            ))
    }

    fn search(&self) -> GraphSearch<'_, ReadView<'a, Store>> {
        GraphSearch::new(&self.meta.graph, &self.view)
    }

    pub(crate) fn search_from(
        &self,
        from: DbId,
        algorithm: SearchQueryAlgorithm,
        limit: u64,
        offset: u64,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, DbError> {
        let search = self.search();
        let depth_first = algorithm == SearchQueryAlgorithm::DepthFirst;
        let elements = !depth_first && algorithm != SearchQueryAlgorithm::BreadthFirst;

        match (limit, offset) {
            (0, 0) => {
                let handler = DefaultHandler::new(self, conditions);

                if elements {
                    search.elements(handler)
                } else if depth_first {
                    search.depth_first_search(from, handler)
                } else {
                    search.breadth_first_search(from, handler)
                }
            }
            (_, 0) => {
                let handler = LimitHandler::new(limit, self, conditions);

                if elements {
                    search.elements(handler)
                } else if depth_first {
                    search.depth_first_search(from, handler)
                } else {
                    search.breadth_first_search(from, handler)
                }
            }
            (0, _) => {
                let handler = OffsetHandler::new(offset, self, conditions);

                if elements {
                    search.elements(handler)
                } else if depth_first {
                    search.depth_first_search(from, handler)
                } else {
                    search.breadth_first_search(from, handler)
                }
            }
            (_, _) => {
                let handler = LimitOffsetHandler::new(limit, offset, self, conditions);

                if elements {
                    search.elements(handler)
                } else if depth_first {
                    search.depth_first_search(from, handler)
                } else {
                    search.breadth_first_search(from, handler)
                }
            }
        }
    }

    pub(crate) fn search_to(
        &self,
        to: DbId,
        algorithm: SearchQueryAlgorithm,
        limit: u64,
        offset: u64,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, DbError> {
        let search = self.search();
        let breadth_first = algorithm == SearchQueryAlgorithm::BreadthFirst;

        match (limit, offset) {
            (0, 0) => {
                let handler = DefaultHandler::new(self, conditions);

                if breadth_first {
                    search.breadth_first_search_reverse(to, handler)
                } else {
                    search.depth_first_search_reverse(to, handler)
                }
            }
            (_, 0) => {
                let handler = LimitHandler::new(limit, self, conditions);

                if breadth_first {
                    search.breadth_first_search_reverse(to, handler)
                } else {
                    search.depth_first_search_reverse(to, handler)
                }
            }
            (0, _) => {
                let handler = OffsetHandler::new(offset, self, conditions);

                if breadth_first {
                    search.breadth_first_search_reverse(to, handler)
                } else {
                    search.depth_first_search_reverse(to, handler)
                }
            }
            (_, _) => {
                let handler = LimitOffsetHandler::new(limit, offset, self, conditions);

                if breadth_first {
                    search.breadth_first_search_reverse(to, handler)
                } else {
                    search.depth_first_search_reverse(to, handler)
                }
            }
        }
    }

    pub(crate) fn search_from_to(
        &self,
        from: DbId,
        to: DbId,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, DbError> {
        self.search()
            .path(from, to, PathHandler::new(self, conditions))
    }

    pub(crate) fn search_to_from(
        &self,
        from: DbId,
        to: DbId,
        conditions: &Vec<QueryCondition>,
    ) -> Result<Vec<DbId>, DbError> {
        self.search()
            .path_reverse(from, to, PathHandler::new(self, conditions))
    }

    pub(crate) fn values(&self, id: DbId) -> Result<Vec<DbKeyValue>, DbError> {
        self.key_values().values(&self.view, &self.meta.keys, id)
    }

    /// The values of the `keys` the element has, in the order of the keys,
    /// as resolved by [`key_ids`](Self::key_ids).
    pub(crate) fn values_by_keys(
        &self,
        id: DbId,
        keys: &[(KeyId, &DbValue)],
    ) -> Result<Vec<DbKeyValue>, DbError> {
        let ids = keys.iter().map(|key| key.0).collect::<Vec<_>>();
        let values = self.key_values().values_by_keys(&self.view, id, &ids)?;

        Ok(keys
            .iter()
            .zip(values)
            .filter_map(|(key, value)| {
                value.map(|value| DbKeyValue {
                    key: key.1.clone(),
                    value,
                })
            })
            .collect())
    }

    /// The value of the `key` of the element.
    pub(crate) fn value(&self, id: DbId, key: &DbValue) -> Result<Option<DbValue>, DbError> {
        match self.meta.keys.id(&self.view, key)? {
            Some(key) => self.key_values().value(&self.view, id, key),
            None => Ok(None),
        }
    }

    pub(crate) fn evaluate_condition(
        &self,
        id: DbId,
        distance: u64,
        condition: &QueryConditionData,
    ) -> Result<SearchControl, DbError> {
        let node = |id: DbId| self.meta.graph.node(&self.view, id);

        Ok(match condition {
            QueryConditionData::Distance(value) => value.compare_distance(distance),
            QueryConditionData::Edge => SearchControl::Continue(id.0 < 0),
            QueryConditionData::EdgeCount(value) => SearchControl::Continue(
                node(id)?.is_some_and(|node| value.compare(node.from_count + node.to_count)),
            ),
            QueryConditionData::EdgeCountFrom(value) => SearchControl::Continue(
                node(id)?.is_some_and(|node| value.compare(node.from_count)),
            ),
            QueryConditionData::EdgeCountTo(value) => {
                SearchControl::Continue(node(id)?.is_some_and(|node| value.compare(node.to_count)))
            }
            QueryConditionData::Ids(values) => {
                let mut found = false;

                for value in values {
                    let other = match value {
                        QueryId::Id(other) => *other,
                        QueryId::Alias(alias) => self.alias_id(alias)?.unwrap_or_default(),
                    };

                    if other == id {
                        found = true;
                        break;
                    }
                }

                SearchControl::Continue(found)
            }
            QueryConditionData::KeyValue(kvc) => SearchControl::Continue(
                self.value(id, &kvc.key)?
                    .is_some_and(|value| kvc.value.compare(&value)),
            ),
            QueryConditionData::Keys(values) => {
                let keys = self.keys(id)?;
                SearchControl::Continue(values.iter().all(|key| keys.contains(key)))
            }
            QueryConditionData::Node => SearchControl::Continue(0 < id.0),
            QueryConditionData::Where(conditions) => {
                self.evaluate_conditions(id, distance, conditions)?
            }
        })
    }

    pub(crate) fn evaluate_conditions(
        &self,
        id: DbId,
        distance: u64,
        conditions: &[QueryCondition],
    ) -> Result<SearchControl, DbError> {
        let mut result = SearchControl::Continue(true);

        for condition in conditions {
            let mut control = self.evaluate_condition(id, distance, &condition.data)?;

            match condition.modifier {
                QueryConditionModifier::Beyond => {
                    if control.is_true() || distance == 0 {
                        control = SearchControl::Continue(result.is_true());
                    } else {
                        control = SearchControl::Stop(result.is_true());
                    }
                }
                QueryConditionModifier::Not => control.flip(),
                QueryConditionModifier::NotBeyond => {
                    if control.is_true() {
                        control = SearchControl::Stop(result.is_true());
                    } else {
                        control = SearchControl::Continue(result.is_true());
                    }
                }
                _ => {}
            };

            result = match condition.logic {
                QueryConditionLogic::And => result.and(control),
                QueryConditionLogic::Or => result.or(control),
            };
        }

        Ok(result)
    }
}

/// The committed database read by an optimization: it has nothing to fix.
impl<S: StorageData> Source for Transaction<'_, S> {
    fn slots(&self) -> u64 {
        self.meta.graph.slots()
    }

    fn graph(&self, start: u64, count: u64) -> Result<Vec<Element>, DbError> {
        let graph = &self.meta.graph;
        let mut elements = Vec::new();

        for slot in start..start + count {
            let id = slot as i64;
            let mut node = graph.node(&self.view, DbId(id))?;
            let mut edge = graph.edge(&self.view, DbId(-id))?;

            if let Some(node) = &mut node {
                node.data = 0;
            }

            if let Some(edge) = &mut edge {
                edge.data = 0;
            }

            elements.push((node, edge));
        }

        Ok(elements)
    }

    fn indexes(&self) -> Result<Vec<DbValue>, DbError> {
        Ok(self
            .meta
            .indexes
            .list()
            .iter()
            .map(|index| index.key.clone())
            .collect())
    }

    fn aliases(&self, _fixed: &mut Vec<String>) -> Result<Vec<(DbId, String)>, DbError> {
        Ok(Transaction::aliases(self)?
            .into_iter()
            .map(|(alias, id)| (id, alias))
            .collect())
    }

    fn value_slots(&self) -> Result<u64, DbError> {
        Ok(self.meta.graph.slots())
    }

    fn values(
        &self,
        start: u64,
        count: u64,
        _fixed: &mut Vec<String>,
    ) -> Result<Vec<(DbId, DbKeyValue)>, DbError> {
        let mut values = Vec::new();

        for (slot, (node, edge)) in (start..).zip(Source::graph(self, start, count)?) {
            let id = slot as i64;

            for (exists, id) in [(node.is_some(), DbId(id)), (edge.is_some(), DbId(-id))] {
                if exists {
                    for key_value in Transaction::values(self, id)? {
                        values.push((id, key_value));
                    }
                }
            }
        }

        Ok(values)
    }
}
