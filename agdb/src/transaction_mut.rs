use crate::Amend;
use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryResult;
use crate::StorageData;
use crate::Transaction;
use crate::collections::key_values::KeyValues;
use crate::collections::keys::ALIAS;
use crate::query::Query;
use crate::query::QueryMut;
use crate::query::query_id::QueryId;
use crate::storage::root::RootMut;
use crate::transaction::ReadView;
use std::borrow::Cow;

/// The `TransactionMut` is a write transaction over the database. It
/// allows running queries via [`exec()`](#method.exec) and
/// [`exec_mut()`](#method.exec_mut). Once a mutable query fails the
/// transaction can no longer run queries nor be committed.
pub struct TransactionMut<'a, Store: StorageData> {
    root: RootMut<'a, Store>,
    failed: Option<DbError>,
}

impl<'a, Store: StorageData> TransactionMut<'a, Store> {
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
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }

        self.read().exec(query)
    }

    /// Executes mutable queries:
    ///
    /// - Insert nodes
    /// - Insert edges
    /// - Insert aliases
    /// - Insert values
    /// - Insert index
    /// - Remove elements
    /// - Remove aliases
    /// - Remove values
    /// - Remove index
    pub fn exec_mut<T: QueryMut>(&mut self, query: T) -> Result<QueryResult, DbError> {
        if let Some(error) = &self.failed {
            return Err(error.clone());
        }

        let result = query.process(self);

        if let Err(error) = &result {
            self.failed = Some(error.clone());
        }

        result
    }

    pub(crate) fn new(root: RootMut<'a, Store>) -> Self {
        Self { root, failed: None }
    }

    /// Commits the transaction unless one of its queries failed: their
    /// writes may be partial so the transaction is dropped instead.
    pub(crate) fn commit(self) -> Result<(), DbError> {
        match self.failed {
            Some(error) => Err(error),
            None => self.root.commit(),
        }
    }

    /// The database as written by the transaction so far.
    pub(crate) fn read(&self) -> Transaction<'_, Store> {
        Transaction::new(
            ReadView::Writing(&self.root.view),
            Cow::Borrowed(&self.root.meta),
        )
    }

    pub(crate) fn db_id(&self, query_id: &QueryId) -> Result<DbId, DbError> {
        self.read().db_id(query_id)
    }

    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn from_id(&self, id: DbId) -> Result<DbId, DbError> {
        self.read().from_id(id)
    }

    pub(crate) fn to_id(&self, id: DbId) -> Result<DbId, DbError> {
        self.read().to_id(id)
    }

    /// Sets the `alias` of the node `id` replacing its current alias
    /// and taking the alias from the node that had it.
    pub(crate) fn insert_alias(&mut self, id: DbId, alias: &String) -> Result<(), DbError> {
        if alias.is_empty() {
            return Err(DbError::query(
                DbErrorType::NotAllowed,
                "Empty alias is not allowed",
            ));
        }

        if id.0 < 0 {
            return Err(DbError::query(
                DbErrorType::NotAllowed,
                format!("Edges cannot have aliases: '{alias}' on '{}'", id.0),
            ));
        }

        let value = DbValue::from(alias.as_str());
        let root = &mut self.root;
        let key_values = KeyValues(&root.meta.graph);
        let old = key_values.value(&root.view, id, ALIAS)?;

        if old.as_ref() == Some(&value) {
            return Ok(());
        }

        if let Some(old) = old {
            root.meta
                .indexes
                .remove(&mut root.view, &mut root.meta.alloc, ALIAS, &old, id)?;
        }

        let owner = root.meta.indexes.aliases().search(&root.view, &value)?;

        if let Some(owner) = owner.first() {
            root.meta.indexes.remove(
                &mut root.view,
                &mut root.meta.alloc,
                ALIAS,
                &value,
                *owner,
            )?;
            key_values.remove(
                &mut root.view,
                &mut root.meta.alloc,
                &mut root.meta.keys,
                *owner,
                ALIAS,
            )?;
        }

        key_values.set(
            &mut root.view,
            &mut root.meta.alloc,
            &mut root.meta.keys,
            id,
            ALIAS,
            &value,
        )?;
        root.meta
            .indexes
            .insert(&mut root.view, &mut root.meta.alloc, ALIAS, &value, id)
    }

    pub(crate) fn insert_edge(&mut self, from: DbId, to: DbId) -> Result<DbId, DbError> {
        let root = &mut self.root;
        root.meta
            .graph
            .insert_edge(&mut root.view, &mut root.meta.alloc, from, to)
    }

    /// Creates the index of the `key` and indexes the values of the key
    /// of all elements.
    pub(crate) fn insert_index(&mut self, key: &DbValue) -> Result<u64, DbError> {
        let root = &mut self.root;

        if root.meta.indexes.index(key).is_some() {
            return Err(DbError::db(
                DbErrorType::NotAllowed,
                format!("Index '{key}' already exists"),
            ));
        }

        root.meta.indexes.insert_index(
            &mut root.view,
            &mut root.meta.alloc,
            &mut root.meta.keys,
            key,
        )?;
        let last = root.meta.indexes.list().len() - 1;
        let key_id = root.meta.indexes.list()[last].key_id;
        let ids = root
            .meta
            .graph
            .elements(&root.view)
            .collect::<Result<Vec<DbId>, DbError>>()?;
        let key_values = KeyValues(&root.meta.graph);

        for id in ids {
            if let Some(value) = key_values.value(&root.view, id, key_id)? {
                root.meta.indexes.insert(
                    &mut root.view,
                    &mut root.meta.alloc,
                    key_id,
                    &value,
                    id,
                )?;
            }
        }

        Ok(root.meta.indexes.list()[last].count)
    }

    pub(crate) fn insert_node(&mut self) -> Result<DbId, DbError> {
        let root = &mut self.root;
        root.meta
            .graph
            .insert_node(&mut root.view, &mut root.meta.alloc)
    }

    /// Adds the `key_value` to the element `id` that does not have the
    /// key yet.
    pub(crate) fn insert_key_value(
        &mut self,
        id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<(), DbError> {
        let root = &mut self.root;
        let key = KeyValues(&root.meta.graph).push(
            &mut root.view,
            &mut root.meta.alloc,
            &mut root.meta.keys,
            id,
            key_value,
        )?;
        root.meta.indexes.insert(
            &mut root.view,
            &mut root.meta.alloc,
            key,
            &key_value.value,
            id,
        )
    }

    pub(crate) fn insert_or_replace_key_value(
        &mut self,
        id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<(), DbError> {
        let root = &mut self.root;
        let (key, old) = KeyValues(&root.meta.graph).insert(
            &mut root.view,
            &mut root.meta.alloc,
            &mut root.meta.keys,
            id,
            key_value,
        )?;

        if let Some(old) = old {
            root.meta
                .indexes
                .remove(&mut root.view, &mut root.meta.alloc, key, &old, id)?;
        }

        root.meta.indexes.insert(
            &mut root.view,
            &mut root.meta.alloc,
            key,
            &key_value.value,
            id,
        )
    }

    pub(crate) fn amend_key_value(
        &mut self,
        id: DbId,
        key_value: &DbKeyValue,
        amend: &Amend,
    ) -> Result<bool, DbError> {
        let existing = self.read().value(id, &key_value.key)?;

        match existing {
            Some(existing_value) => {
                let merged_value = match amend {
                    Amend::Add => existing_value.amend_add(&key_value.value)?,
                    Amend::Remove => existing_value.amend_remove(&key_value.value)?,
                    Amend::AddBitwise(op) => {
                        existing_value.amend_bitwise(&key_value.value, op, true)?
                    }
                    Amend::RemoveBitwise(op) => {
                        existing_value.amend_bitwise(&key_value.value, op, false)?
                    }
                    Amend::None => unreachable!(),
                };

                let merged_kv = DbKeyValue {
                    key: key_value.key.clone(),
                    value: merged_value,
                };
                self.insert_or_replace_key_value(id, &merged_kv)?;
                Ok(true)
            }
            None => match amend {
                Amend::Add | Amend::AddBitwise(_) => {
                    self.insert_or_replace_key_value(id, key_value)?;
                    Ok(true)
                }
                Amend::Remove | Amend::RemoveBitwise(_) => Ok(false),
                Amend::None => unreachable!(),
            },
        }
    }

    pub(crate) fn remove(&mut self, query_id: &QueryId) -> Result<bool, DbError> {
        let id = match query_id {
            QueryId::Id(id) => Some(*id),
            QueryId::Alias(alias) => self.read().alias_id(alias)?,
        };

        match id {
            Some(id) => self.remove_id(id),
            None => Ok(false),
        }
    }

    /// Removes the element `id` with its values and, if it is a node,
    /// with its edges.
    pub(crate) fn remove_id(&mut self, id: DbId) -> Result<bool, DbError> {
        if !self.read().exists(id)? {
            return Ok(false);
        }

        if 0 < id.0 {
            for edge in self.node_edges(id)? {
                self.remove_all_values(edge)?;
            }

            self.remove_all_values(id)?;
            let root = &mut self.root;
            root.meta.graph.remove_node(&mut root.view, id)?;
        } else {
            self.remove_all_values(id)?;
            let root = &mut self.root;
            root.meta.graph.remove_edge(&mut root.view, id)?;
        }

        Ok(true)
    }

    pub(crate) fn remove_alias(&mut self, alias: &str) -> Result<bool, DbError> {
        let value = DbValue::from(alias);
        let root = &mut self.root;
        let owner = root.meta.indexes.aliases().search(&root.view, &value)?;

        let Some(id) = owner.first() else {
            return Ok(false);
        };

        root.meta
            .indexes
            .remove(&mut root.view, &mut root.meta.alloc, ALIAS, &value, *id)?;
        KeyValues(&root.meta.graph).remove(
            &mut root.view,
            &mut root.meta.alloc,
            &mut root.meta.keys,
            *id,
            ALIAS,
        )?;

        Ok(true)
    }

    pub(crate) fn remove_index(&mut self, key: &DbValue) -> Result<u64, DbError> {
        let root = &mut self.root;
        root.meta.indexes.remove_index(
            &mut root.view,
            &mut root.meta.alloc,
            &mut root.meta.keys,
            key,
        )
    }

    pub(crate) fn reserve_key_value_capacity(
        &mut self,
        id: DbId,
        additional: u64,
    ) -> Result<(), DbError> {
        let root = &mut self.root;
        KeyValues(&root.meta.graph).reserve(&mut root.view, &mut root.meta.alloc, id, additional)
    }

    /// Removes the values of the `keys` from the element `id` and returns
    /// how many it had.
    pub(crate) fn remove_keys(&mut self, id: DbId, keys: &[DbValue]) -> Result<u64, DbError> {
        let mut count = 0;

        let key_ids = self.read().key_ids(keys)?;

        for (key, _) in key_ids {
            let root = &mut self.root;
            let key_values = KeyValues(&root.meta.graph);

            while let Some(value) = key_values.remove(
                &mut root.view,
                &mut root.meta.alloc,
                &mut root.meta.keys,
                id,
                key,
            )? {
                root.meta
                    .indexes
                    .remove(&mut root.view, &mut root.meta.alloc, key, &value, id)?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// The edges from the node `id`, then those to it that are not also
    /// from it.
    fn node_edges(&self, id: DbId) -> Result<Vec<DbId>, DbError> {
        let graph = &self.root.meta.graph;
        let view = &self.root.view;
        let (mut from, mut to) = graph
            .node(view, id)?
            .map_or((DbId(0), DbId(0)), |node| (node.first_from, node.first_to));
        let mut edges = Vec::new();

        while let Some(edge) = graph.edge(view, from)? {
            edges.push(from);
            from = edge.next_from;
        }

        while let Some(edge) = graph.edge(view, to)? {
            if edge.from != id {
                edges.push(to);
            }

            to = edge.next_to;
        }

        Ok(edges)
    }

    /// Removes the values of the element `id`, its alias included, and
    /// their index entries.
    fn remove_all_values(&mut self, id: DbId) -> Result<(), DbError> {
        let root = &mut self.root;
        let removed = KeyValues(&root.meta.graph).remove_all(
            &mut root.view,
            &mut root.meta.alloc,
            &mut root.meta.keys,
            id,
        )?;

        for (key, value) in removed {
            root.meta
                .indexes
                .remove(&mut root.view, &mut root.meta.alloc, key, &value, id)?;
        }

        Ok(())
    }
}
