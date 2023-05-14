use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::query_values::QueryValues;
use super::QueryMut;
use crate::commands_mut::CommandsMut;
use crate::Db;
use crate::DbElement;
use crate::DbId;
use crate::QueryError;
use crate::QueryResult;

pub struct InsertEdgesQuery {
    pub from: QueryIds,
    pub to: QueryIds,
    pub values: QueryValues,
    pub each: bool,
}

impl QueryMut for InsertEdgesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        todo!()
    }

    fn process(&self, db: &mut Db, result: &mut QueryResult) -> Result<(), QueryError> {
        match &self.from {
            QueryIds::Ids(from) => match &self.to {
                QueryIds::Ids(to) => {
                    let ids = if self.each || from.len() != to.len() {
                        many_to_many_each(db, from, to)?
                    } else {
                        many_to_many(db, from, to)?
                    };
                    result.result = ids.len() as i64;
                    result.elements = ids
                        .into_iter()
                        .map(|id| DbElement {
                            index: id,
                            values: vec![],
                        })
                        .collect();

                    Ok(())
                }
                QueryIds::Search(_) => Err(QueryError::from("Invalid insert edges query")),
            },
            QueryIds::Search(_) => Err(QueryError::from("Invalid insert edges query")),
        }
    }
}

fn many_to_many(db: &mut Db, from: &[QueryId], to: &[QueryId]) -> Result<Vec<DbId>, QueryError> {
    let mut ids = vec![];

    for (from, to) in from.iter().zip(to.iter()) {
        ids.push(db.insert_edge(from, to)?);
    }

    Ok(ids)
}

fn many_to_many_each(
    db: &mut Db,
    from: &[QueryId],
    to: &[QueryId],
) -> Result<Vec<DbId>, QueryError> {
    let mut ids = vec![];

    for from in from {
        for to in to {
            ids.push(db.insert_edge(from, to)?);
        }
    }

    Ok(ids)
}
