use super::TransactionMut;
use crate::Amend;
use crate::DbError;
use crate::DbErrorType;
use crate::DbId;
use crate::DbKeyValue;
use crate::DbValue;
use crate::StorageData;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Action {
    InsertNode(DbId),
    InsertEdge(DbId, DbId, DbId),
    InsertAlias(DbId, String),
    InsertValue(DbId, DbKeyValue),
    ReplaceValue(DbId, DbKeyValue),
    AmendValue(DbId, DbKeyValue, Amend),
    ReserveValues(DbId, u64),
    Remove(DbId),
    RemoveAlias(String),
    RemoveKeys(DbId, Vec<DbValue>),
    InsertIndex(DbValue),
    RemoveIndex(DbValue),
}

impl<Store: StorageData> TransactionMut<'_, Store> {
    pub(super) fn log(&mut self, action: impl FnOnce() -> Action) {
        if let Some(log) = &mut self.log {
            log.push(action());
        }
    }

    pub(crate) fn replay(
        &mut self,
        log: &[Action],
        version: u64,
        removed: &HashMap<DbId, u64>,
    ) -> Result<(), DbError> {
        let mut own = HashSet::new();

        for action in log {
            // WHY: an element removed after the snapshot is gone, while one the
            // transaction created is alive whatever was removed before.
            let alive = |transaction: &Self, id: DbId| -> Result<bool, DbError> {
                Ok(own.contains(&id)
                    || (removed.get(&id).is_none_or(|removed| *removed <= version)
                        && transaction.read().exists(id)?))
            };
            // WHY: changing an element removed concurrently conflicts, the removal
            // would otherwise silently lose the change.
            let changed = |transaction: &Self, id: DbId| -> Result<(), DbError> {
                if alive(transaction, id)? {
                    Ok(())
                } else {
                    Err(DbError::db(
                        DbErrorType::Conflict,
                        format!("Element '{}' was removed by a concurrent transaction", id.0),
                    ))
                }
            };

            match action {
                Action::InsertNode(id) => {
                    let root = &mut self.root;
                    root.meta
                        .graph
                        .insert_node_at(&mut root.view, &mut root.meta.alloc, *id)?;
                    own.insert(*id);
                }
                Action::InsertEdge(id, from, to) => {
                    // WHY: an edge needs both endpoints not removed after the snapshot.
                    changed(self, *from)?;
                    changed(self, *to)?;
                    let root = &mut self.root;
                    root.meta.graph.insert_edge_at(
                        &mut root.view,
                        &mut root.meta.alloc,
                        *id,
                        *from,
                        *to,
                    )?;
                    own.insert(*id);
                }
                Action::InsertAlias(id, alias) => {
                    changed(self, *id)?;
                    self.insert_alias(*id, alias)?;
                }
                // WHY: a plain insert only for elements created in the transaction,
                // elsewhere the key may exist now so the last commit wins.
                Action::InsertValue(id, key_value) if own.contains(id) => {
                    self.insert_key_value(*id, key_value)?;
                }
                Action::InsertValue(id, key_value) | Action::ReplaceValue(id, key_value) => {
                    changed(self, *id)?;
                    self.insert_or_replace_key_value(*id, key_value)?;
                }
                Action::AmendValue(id, key_value, amend) => {
                    changed(self, *id)?;
                    self.amend_key_value(*id, key_value, amend)?;
                }
                Action::ReserveValues(id, additional) => {
                    if alive(self, *id)? {
                        self.reserve_key_value_capacity(*id, *additional)?;
                    }
                }
                Action::Remove(id) => {
                    // WHY: a concurrent removal already did it, which is not an error.
                    if alive(self, *id)? {
                        self.remove_id(*id)?;
                    }
                }
                Action::RemoveAlias(alias) => {
                    self.remove_alias(alias)?;
                }
                Action::RemoveKeys(id, keys) => {
                    if alive(self, *id)? {
                        self.remove_keys(*id, keys)?;
                    }
                }
                Action::InsertIndex(key) => {
                    // WHY: unlike values, an index has no last writer wins, the first one stays.
                    if self.root.meta.indexes.index(key).is_some() {
                        return Err(DbError::db(
                            DbErrorType::Conflict,
                            format!("Index '{key}' was created by a concurrent transaction"),
                        ));
                    }

                    self.insert_index(key)?;
                }
                Action::RemoveIndex(key) => {
                    self.remove_index(key)?;
                }
            }
        }

        Ok(())
    }
}
