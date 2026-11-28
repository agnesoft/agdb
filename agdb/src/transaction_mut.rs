use crate::Amend;
use crate::DbElement;
use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::QueryResult;
use crate::StorageData;
use crate::SyncMode;
use crate::Transaction;
use crate::collections::key_values::KeyValues;
use crate::collections::keys::ALIAS;
use crate::db::id_pool::IdPool;
use crate::db::id_pool::Reserved;
use crate::query::Query;
use crate::query::QueryMut;
use crate::query::query_id::QueryId;
use crate::storage::pages::Overlay;
use crate::storage::root::Meta;
use crate::storage::root::RootMut;
use crate::transaction::ReadView;
use std::borrow::Cow;

mod log;

pub(crate) use log::Action;

/// The `TransactionMut` is a write transaction over the database. It
/// allows running queries via [`exec()`](#method.exec) and
/// [`exec_mut()`](#method.exec_mut). Once a mutable query fails the
/// transaction can no longer run queries nor be committed.
pub struct TransactionMut<'a, Store: StorageData> {
    root: RootMut<'a, Store>,
    failed: Option<DbError>,
    ids: Option<Ids<'a>>,
    log: Option<Vec<Action>>,
}

pub(crate) struct Ids<'a> {
    pool: &'a IdPool,
    state: IdState,
}

#[derive(Debug, Default)]
struct IdState {
    version: u64,
    generation: u64,
    // WHY: only an exclusive transaction can reuse the ids it freed.
    reuse: bool,
    taken: [Vec<Reserved>; 2],
    spare: [Vec<Reserved>; 2],
    freed: [Vec<u64>; 2],
    removed: Vec<DbId>,
    committed: Option<u64>,
}

impl<'a> Ids<'a> {
    pub(crate) fn version(&self) -> u64 {
        self.state.version
    }

    pub(crate) fn generation(&self) -> u64 {
        self.state.generation
    }

    fn new(pool: &'a IdPool, version: u64, generation: u64, reuse: bool) -> Self {
        Self {
            pool,
            state: IdState {
                version,
                generation,
                reuse,
                ..IdState::default()
            },
        }
    }

    fn freeing<'b>(ids: &'b mut Option<Self>) -> impl FnMut(DbId) + 'b {
        move |id| {
            if let Some(ids) = ids {
                ids.state.freed[usize::from(id.0 < 0)].push(id.0.unsigned_abs());
                ids.state.removed.push(id);
            }
        }
    }

    fn take(
        &mut self,
        edges: bool,
        next_free: impl FnMut(u64) -> Result<u64, DbError>,
    ) -> Result<u64, DbError> {
        let kind = usize::from(edges);
        let state = &mut self.state;

        if state.reuse
            && let Some(slot) = state.freed[kind].pop()
        {
            return Ok(slot);
        }

        if state.spare[kind].is_empty() {
            // WHY: batches grow with the slots taken so a transaction inserting
            // many elements locks the pool seldom.
            let count = state.taken[kind].len().clamp(1, MAX_RESERVED);
            self.pool.reserve(
                edges,
                (state.version, state.generation),
                count,
                next_free,
                &mut state.spare[kind],
            )?;
        }

        let reserved = state.spare[kind].pop().expect("reserved a slot");
        state.taken[kind].push(reserved);
        Ok(reserved.0)
    }
}

const MAX_RESERVED: usize = 64;

impl Drop for Ids<'_> {
    fn drop(&mut self) {
        let state = &mut self.state;

        // WHY: the slots taken by a transaction that did not commit go back to the pool.
        if state.committed.is_none() {
            for (spare, taken) in state.spare.iter_mut().zip(&mut state.taken) {
                spare.extend(taken.drain(..).rev());
            }
        }

        let freed = state
            .committed
            .map(|version| ([&state.freed[0][..], &state.freed[1][..]], version));
        self.pool
            .finish(state.generation, [&state.spare[0], &state.spare[1]], freed);
    }
}

pub(crate) struct Pending<'a> {
    pub(crate) failed: Option<DbError>,
    pub(crate) log: Vec<Action>,
    pub(crate) ids: Ids<'a>,
    pub(crate) lens: (u64, u64),
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
        Self {
            root,
            failed: None,
            ids: None,
            log: None,
        }
    }

    pub(crate) fn with_ids(mut self, pool: &'a IdPool) -> Self {
        pool.init(&self.root.meta.graph);
        self.ids = Some(Ids::new(pool, u64::MAX, pool.generation(), true));
        self
    }

    pub(crate) fn concurrent(
        root: RootMut<'a, Store>,
        pool: &'a IdPool,
        version: u64,
        generation: u64,
    ) -> Self {
        Self {
            root,
            failed: None,
            ids: Some(Ids::new(pool, version, generation, false)),
            log: Some(Vec::new()),
        }
    }

    pub(crate) fn committing(mut self, mut ids: Ids<'a>, replay: bool) -> Self {
        // WHY: a replay redoes the removals, so the ids freed over the snapshot are forgotten.
        if replay {
            for freed in &mut ids.state.freed {
                freed.clear();
            }

            ids.state.removed.clear();
        }

        self.ids = Some(ids);
        self
    }

    pub(crate) fn into_pending<'r>(mut self, pool: &'r IdPool) -> Pending<'r> {
        let state = self
            .ids
            .take()
            .map(|mut ids| std::mem::take(&mut ids.state))
            .unwrap_or_default();

        Pending {
            failed: self.failed.take(),
            log: self.log.take().unwrap_or_default(),
            ids: Ids { pool, state },
            lens: self.root.view.lens(),
        }
    }

    pub(crate) fn adopt(&mut self, overlay: &mut Overlay, lens: (u64, u64), meta: &Meta) {
        self.root.adopt(overlay, lens, meta);
    }

    pub(crate) fn written(&self) -> Option<(u64, SyncMode)> {
        // WHY: `None` for a failed query, the transaction will not commit.
        self.failed.is_none().then(|| self.root.view.written())
    }

    pub(crate) fn removed(&self) -> &[DbId] {
        self.ids.as_ref().map_or(&[], |ids| &ids.state.removed[..])
    }

    pub(crate) fn commit(mut self, version: u64) -> Result<(), DbError> {
        // WHY: the writes of a failed query may be partial so it is dropped instead.
        if let Some(error) = self.failed {
            return Err(error);
        }

        let committed = self.root.commit();

        if let Some(ids) = &mut self.ids {
            match &committed {
                Ok(()) => ids.state.committed = Some(version),
                // WHY: the storage may hold either state after a failed commit,
                // so the pool is made again from it.
                Err(_) => ids.pool.reset(),
            }
        }

        committed
    }

    /// The database as written by the transaction so far.
    pub(crate) fn read(&self) -> Transaction<'_, Store> {
        Transaction::new(
            ReadView::Writing(&self.root.view),
            Cow::Borrowed(self.root.meta),
        )
    }

    pub(crate) fn db_id(&self, query_id: &QueryId) -> Result<DbId, DbError> {
        self.read().db_id(query_id)
    }

    pub(crate) fn element(&self, id: DbId, values: Vec<DbKeyValue>) -> Result<DbElement, DbError> {
        self.read().element(id, values)
    }

    /// Sets the `alias` of the node `id` replacing its current alias
    /// and taking the alias from the node that had it.
    pub(crate) fn insert_alias(&mut self, id: DbId, alias: &String) -> Result<(), DbError> {
        self.set_alias(id, alias)?;
        self.log(|| Action::InsertAlias(id, alias.clone()));
        Ok(())
    }

    fn set_alias(&mut self, id: DbId, alias: &String) -> Result<(), DbError> {
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
        let graph = &mut root.meta.graph;

        let Some(ids) = &mut self.ids else {
            return graph.insert_edge(&mut root.view, &mut root.meta.alloc, from, to);
        };

        let id = DbId(-(ids.take(true, |slot| graph.next_free(&root.view, true, slot))? as i64));
        graph.insert_edge_at(&mut root.view, &mut root.meta.alloc, id, from, to)?;
        self.log(|| Action::InsertEdge(id, from, to));
        Ok(id)
    }

    /// Creates the index of the `key` and indexes the values of the key
    /// of all elements.
    pub(crate) fn insert_index(&mut self, key: &DbValue) -> Result<u64, DbError> {
        let count = self.create_index(key)?;
        self.log(|| Action::InsertIndex(key.clone()));
        Ok(count)
    }

    fn create_index(&mut self, key: &DbValue) -> Result<u64, DbError> {
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
        let key_values = KeyValues(&root.meta.graph);

        const BATCH: usize = 4096;
        let mut slot = 1_u64;
        let mut in_edges = false;

        loop {
            let batch: Vec<DbId> = {
                let mut iter = root.meta.graph.elements_from(&root.view, slot, in_edges);
                let ids: Vec<DbId> = iter.by_ref().take(BATCH).collect::<Result<Vec<_>, _>>()?;
                in_edges = iter.edges;
                slot = iter.slot;
                ids
            };

            if batch.is_empty() {
                break;
            }

            for id in batch {
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
        }

        Ok(root.meta.indexes.list()[last].count)
    }

    pub(crate) fn insert_node(&mut self) -> Result<DbId, DbError> {
        let root = &mut self.root;
        let graph = &mut root.meta.graph;

        let Some(ids) = &mut self.ids else {
            return graph.insert_node(&mut root.view, &mut root.meta.alloc);
        };

        let id = DbId(ids.take(false, |slot| graph.next_free(&root.view, false, slot))? as i64);
        graph.insert_node_at(&mut root.view, &mut root.meta.alloc, id)?;
        self.log(|| Action::InsertNode(id));
        Ok(id)
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
        )?;
        self.log(|| Action::InsertValue(id, key_value.clone()));
        Ok(())
    }

    pub(crate) fn insert_or_replace_key_value(
        &mut self,
        id: DbId,
        key_value: &DbKeyValue,
    ) -> Result<(), DbError> {
        self.replace_value(id, key_value)?;
        self.log(|| Action::ReplaceValue(id, key_value.clone()));
        Ok(())
    }

    fn replace_value(&mut self, id: DbId, key_value: &DbKeyValue) -> Result<(), DbError> {
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
        let amended = self.amend_value(id, key_value, amend)?;
        self.log(|| Action::AmendValue(id, key_value.clone(), amend.clone()));
        Ok(amended)
    }

    fn amend_value(
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
                self.replace_value(id, &merged_kv)?;
                Ok(true)
            }
            None => match amend {
                Amend::Add | Amend::AddBitwise(_) => {
                    self.replace_value(id, key_value)?;
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

        self.remove_element(id)?;
        self.log(|| Action::Remove(id));
        Ok(true)
    }

    fn remove_element(&mut self, id: DbId) -> Result<(), DbError> {
        if 0 < id.0 {
            for edge in self.node_edges(id)? {
                self.remove_all_values(edge)?;
            }

            self.remove_all_values(id)?;
            let freed = Ids::freeing(&mut self.ids);
            let root = &mut self.root;
            root.meta.graph.remove_node(&mut root.view, id, freed)?;
        } else {
            self.remove_all_values(id)?;
            let freed = Ids::freeing(&mut self.ids);
            let root = &mut self.root;
            root.meta.graph.remove_edge(&mut root.view, id, freed)?;
        }

        Ok(())
    }

    pub(crate) fn remove_alias(&mut self, alias: &str) -> Result<bool, DbError> {
        let removed = self.take_alias(alias)?;
        self.log(|| Action::RemoveAlias(alias.to_string()));
        Ok(removed)
    }

    fn take_alias(&mut self, alias: &str) -> Result<bool, DbError> {
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
        let count = root.meta.indexes.remove_index(
            &mut root.view,
            &mut root.meta.alloc,
            &mut root.meta.keys,
            key,
        )?;
        self.log(|| Action::RemoveIndex(key.clone()));
        Ok(count)
    }

    pub(crate) fn reserve_key_value_capacity(
        &mut self,
        id: DbId,
        additional: u64,
    ) -> Result<(), DbError> {
        let root = &mut self.root;
        KeyValues(&root.meta.graph).reserve(
            &mut root.view,
            &mut root.meta.alloc,
            id,
            additional,
        )?;
        self.log(|| Action::ReserveValues(id, additional));
        Ok(())
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

        self.log(|| Action::RemoveKeys(id, keys.to_vec()));
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
