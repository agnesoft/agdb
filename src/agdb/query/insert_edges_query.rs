use super::query_id::QueryId;
use super::query_ids::QueryIds;
use super::query_values::QueryValues;
use super::QueryMut;
use crate::commands_mut::insert_edge::InsertEdge;
use crate::commands_mut::CommandsMut;
use crate::QueryError;

pub struct InsertEdgesQuery {
    pub from: QueryIds,
    pub to: QueryIds,
    pub values: QueryValues,
    pub each: bool,
}

impl QueryMut for InsertEdgesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        match &self.from {
            QueryIds::Ids(ids) => {
                if self.each {
                    self.many_to_many_each(ids)
                } else {
                    self.many_to_many(ids)
                }
            }
            QueryIds::Search(_) => Err(QueryError::from("Invalid insert edges query")),
        }
    }
}

impl InsertEdgesQuery {
    fn many_to_many(&self, from: &[QueryId]) -> Result<Vec<CommandsMut>, QueryError> {
        let mut commands = Vec::<CommandsMut>::new();

        match &self.to {
            QueryIds::Ids(ids) => {
                for (from, to) in from.iter().zip(ids.iter()) {
                    commands.push(CommandsMut::InsertEdge(InsertEdge::new(
                        from.clone(),
                        to.clone(),
                    )));
                }
            }
            QueryIds::Search(_) => return Err(QueryError::from("Invalid insert edges query")),
        }

        Ok(commands)
    }

    fn many_to_many_each(&self, from: &Vec<QueryId>) -> Result<Vec<CommandsMut>, QueryError> {
        let mut commands = Vec::<CommandsMut>::new();

        match &self.to {
            QueryIds::Ids(ids) => {
                for from in from {
                    for to in ids {
                        commands.push(CommandsMut::InsertEdge(InsertEdge::new(
                            from.clone(),
                            to.clone(),
                        )));
                    }
                }
            }
            QueryIds::Search(_) => return Err(QueryError::from("Invalid insert edges query")),
        }

        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::search_query::SearchQuery;

    #[test]
    fn one_to_one() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(1)]),
            to: QueryIds::Ids(vec![QueryId::from(2)]),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![CommandsMut::InsertEdge(InsertEdge::new(
                QueryId::from(1),
                QueryId::from(2)
            )),]
        );
    }

    #[test]
    fn alias_to_alias() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::Alias("alias".to_string())]),
            to: QueryIds::Ids(vec![QueryId::Alias("alias2".to_string())]),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![CommandsMut::InsertEdge(InsertEdge::new(
                QueryId::Alias("alias".to_string()),
                QueryId::Alias("alias2".to_string()),
            )),]
        );
    }

    #[test]
    fn one_to_many() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(1)]),
            to: QueryIds::Ids(vec![QueryId::from(2), QueryId::from(3)]),
            values: QueryValues::None,
            each: true,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(1), QueryId::from(2))),
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(1), QueryId::from(3))),
            ]
        );
    }

    #[test]
    fn many_to_one() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(1), QueryId::from(2)]),
            to: QueryIds::Ids(vec![QueryId::from(3)]),
            values: QueryValues::None,
            each: true,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(1), QueryId::from(3))),
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(2), QueryId::from(3))),
            ]
        );
    }

    #[test]
    fn many_to_each() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(1), QueryId::from(2)]),
            to: QueryIds::Ids(vec![QueryId::from(3)]),
            values: QueryValues::None,
            each: true,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(1), QueryId::from(3))),
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(2), QueryId::from(3))),
            ]
        );
    }

    #[test]
    fn many_to_many() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(1), QueryId::from(2)]),
            to: QueryIds::Ids(vec![QueryId::from(3), QueryId::from(4)]),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(1), QueryId::from(3))),
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(2), QueryId::from(4))),
            ]
        );
    }

    #[test]
    fn many_to_many_each() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(1), QueryId::from(2)]),
            to: QueryIds::Ids(vec![QueryId::from(3), QueryId::from(4)]),
            values: QueryValues::None,
            each: true,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(1), QueryId::from(3))),
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(1), QueryId::from(4))),
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(2), QueryId::from(3))),
                CommandsMut::InsertEdge(InsertEdge::new(QueryId::from(2), QueryId::from(4))),
            ]
        );
    }

    #[test]
    fn invalid_query_preprocessing_from() {
        let query = InsertEdgesQuery {
            from: QueryIds::Search(SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
            }),
            to: QueryIds::Ids(vec![QueryId::from(2)]),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid insert edges query").description
        );
    }

    #[test]
    fn invalid_query_preprocessing_to() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(2)]),
            to: QueryIds::Search(SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
            }),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid insert edges query").description
        );
    }

    #[test]
    fn invalid_query_preprocessing_many_each() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(2)]),
            to: QueryIds::Search(SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
            }),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid insert edges query").description
        );
    }

    #[test]
    fn invalid_query_preprocessing_many_many() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(2)]),
            to: QueryIds::Search(SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
            }),
            values: QueryValues::None,
            each: true,
        };

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid insert edges query").description
        );
    }
}
