use crate::db_pool::ErrorCode;
use crate::db_pool::ServerError;
use crate::server_error::ServerResult;
use crate::utilities::log_migration;
use agdb::DbAny;
use agdb::DbAnyTransaction;
use agdb::DbAnyTransactionMut;
use agdb::DbError;
use agdb::QueryConditionData;
use agdb::QueryId;
use agdb::QueryIds;
use agdb::QueryResult;
use agdb::QueryType;
use agdb::SearchQuery;
use agdb::StorageConfig;
use agdb_api::DbKind;
use agdb_api::Queries;
use agdb_api::QueryAudit;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[derive(Clone)]
pub(crate) struct UserDb(pub(crate) Arc<DbAny>);

impl UserDb {
    pub(crate) fn new(name: &str, db_type: DbKind, config: StorageConfig) -> ServerResult<Self> {
        let db = match db_type {
            DbKind::Memory => DbAny::new_memory(name)?,
            DbKind::File => DbAny::new_file(name)?,
            DbKind::Mapped => DbAny::new_mapped(name)?,
        };
        db.set_storage_config(config);
        log_migration(name, &db);
        Ok(Self(Arc::new(db)))
    }

    pub(crate) fn exec(&self, mut queries: Queries) -> ServerResult<Vec<QueryResult>> {
        self.0.transaction(|t| {
            let mut results = vec![];

            for q in queries.0.iter_mut() {
                let result = t_exec(t, q, &results)?;
                results.push(result);
            }

            Ok(results)
        })
    }

    pub(crate) fn exec_mut(
        &self,
        mut queries: Queries,
        username: &str,
    ) -> ServerResult<(Vec<QueryResult>, Vec<QueryAudit>)> {
        self.0.transaction_mut(|t| {
            let mut audit = vec![];
            let mut results = vec![];
            let mut qs = vec![];
            std::mem::swap(&mut queries.0, &mut qs);

            for q in qs {
                let result = t_exec_mut(t, q, &results, &mut audit, username)?;
                results.push(result);
            }

            Ok((results, audit))
        })
    }

    pub(crate) fn optimize_storage(&self) -> ServerResult<()> {
        self.0.optimize_storage()?;
        Ok(())
    }

    pub(crate) fn rename(&self, target_name: &str) -> ServerResult<()> {
        self.0.rename(target_name)?;
        Ok(())
    }

    pub(crate) fn size(&self) -> u64 {
        self.0.size()
    }
}

fn t_exec(
    t: &DbAnyTransaction,
    q: &mut QueryType,
    results: &[QueryResult],
) -> ServerResult<QueryResult> {
    match q {
        QueryType::Search(q) => {
            inject_results_search(q, results)?;
            t.exec(&*q)
        }
        QueryType::SelectAliases(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(&*q)
        }
        QueryType::SelectAllAliases(q) => t.exec(&*q),
        QueryType::SelectAllEdgeCount(q) => t.exec(&*q),
        QueryType::SelectEdgeCount(q) => t.exec(&*q),
        QueryType::SelectIndexes(q) => t.exec(&*q),
        QueryType::SelectKeys(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(&*q)
        }
        QueryType::SelectKeyCount(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(&*q)
        }
        QueryType::SelectNodeCount(q) => t.exec(&*q),
        QueryType::SelectValues(q) => {
            inject_results(&mut q.ids, results)?;
            t.exec(&*q)
        }
        _ => Err(DbError::query(
            agdb::DbErrorType::NotAllowed,
            "mutable query not allowed",
        )),
    }
    .map_err(|e| ServerError::new(ErrorCode::DbError.into(), &e.description))
}

fn audit_query(user: &str, audit: &mut Vec<QueryAudit>, query: QueryType) {
    audit.push(QueryAudit {
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        username: user.to_string(),
        query,
    });
}

fn t_exec_mut(
    t: &mut DbAnyTransactionMut,
    mut q: QueryType,
    results: &[QueryResult],
    audit: &mut Vec<QueryAudit>,
    username: &str,
) -> ServerResult<QueryResult> {
    let mut do_audit = false;

    let r = match &mut q {
        QueryType::Search(q) => {
            inject_results_search(q, results)?;
            t.exec(&*q)
        }
        QueryType::SelectAliases(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(&*q)
        }
        QueryType::SelectAllAliases(q) => t.exec(&*q),
        QueryType::SelectAllEdgeCount(q) => t.exec(&*q),
        QueryType::SelectEdgeCount(q) => t.exec(&*q),
        QueryType::SelectIndexes(q) => t.exec(&*q),
        QueryType::SelectKeys(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(&*q)
        }
        QueryType::SelectKeyCount(q) => {
            inject_results(&mut q.0, results)?;
            t.exec(&*q)
        }
        QueryType::SelectNodeCount(q) => t.exec(&*q),
        QueryType::SelectValues(q) => {
            inject_results(&mut q.ids, results)?;
            t.exec(&*q)
        }
        QueryType::InsertAlias(q) => {
            do_audit = true;
            inject_results(&mut q.ids, results)?;
            t.exec_mut(&*q)
        }
        QueryType::InsertEdges(q) => {
            do_audit = true;
            inject_results(&mut q.ids, results)?;
            inject_results(&mut q.from, results)?;
            inject_results(&mut q.to, results)?;

            t.exec_mut(&*q)
        }
        QueryType::InsertNodes(q) => {
            do_audit = true;
            inject_results(&mut q.ids, results)?;
            t.exec_mut(&*q)
        }
        QueryType::InsertValues(q) => {
            do_audit = true;
            inject_results(&mut q.ids, results)?;
            t.exec_mut(&*q)
        }
        QueryType::Remove(q) => {
            do_audit = true;
            inject_results(&mut q.0, results)?;
            t.exec_mut(&*q)
        }
        QueryType::InsertIndex(q) => {
            do_audit = true;
            t.exec_mut(&*q)
        }
        QueryType::RemoveAliases(q) => {
            do_audit = true;
            t.exec_mut(&*q)
        }
        QueryType::RemoveIndex(q) => {
            do_audit = true;
            t.exec_mut(&*q)
        }
        QueryType::RemoveValues(q) => {
            do_audit = true;
            inject_results(&mut q.0.ids, results)?;
            t.exec_mut(&*q)
        }
    };

    if do_audit {
        audit_query(username, audit, q);
    }

    r.map_err(|e| ServerError::new(ErrorCode::DbError.into(), &e.description))
}

fn id_or_result(id: QueryId, results: &[QueryResult]) -> ServerResult<QueryId> {
    if let QueryId::Alias(alias) = &id
        && let Some(index) = alias.strip_prefix(':')
        && let Ok(index) = index.parse::<usize>()
    {
        return Ok(QueryId::Id(
            results
                .get(index)
                .ok_or(ServerError::new(
                    ErrorCode::DbError.into(),
                    &format!(
                        "Results index out of bounds '{index}' (> {})",
                        results.len()
                    ),
                ))?
                .elements
                .first()
                .ok_or(ServerError::new(
                    ErrorCode::DbError.into(),
                    "No element found in the result",
                ))?
                .id,
        ));
    }

    Ok(id)
}

fn inject_results(ids: &mut QueryIds, results: &[QueryResult]) -> ServerResult<()> {
    match ids {
        QueryIds::Ids(ids) => inject_results_ids(ids, results),
        QueryIds::Search(search) => inject_results_search(search, results),
    }
    .map_err(|mut e| {
        e.status = ErrorCode::DbError.into();
        e
    })
}

fn inject_results_search(search: &mut SearchQuery, results: &[QueryResult]) -> ServerResult<()> {
    search.origin = id_or_result(search.origin.clone(), results)?;
    search.destination = id_or_result(search.destination.clone(), results)?;

    for c in &mut search.conditions {
        if let QueryConditionData::Ids(ids) = &mut c.data {
            inject_results_ids(ids, results)?;
        }
    }

    Ok(())
}

fn inject_results_ids(ids: &mut Vec<QueryId>, results: &[QueryResult]) -> ServerResult<()> {
    for i in (0..ids.len()).rev() {
        if let QueryId::Alias(alias) = &ids[i]
            && let Some(index) = alias.strip_prefix(':')
            && let Ok(index) = index.parse::<usize>()
        {
            let result_ids = results
                .get(index)
                .ok_or(ServerError::new(
                    ErrorCode::DbError.into(),
                    &format!(
                        "Results index out of bounds '{index}' (> {})",
                        results.len()
                    ),
                ))?
                .ids()
                .into_iter()
                .map(QueryId::Id)
                .collect::<Vec<QueryId>>();
            ids.splice(i..i + 1, result_ids);
        }
    }

    Ok(())
}
