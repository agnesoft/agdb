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
            QueryIds::Id(id) => self.one_to_many(id),
            QueryIds::Ids(ids) => {
                if self.each {
                    self.many_to_many_each(ids)
                } else {
                    self.many_to_many(ids)
                }
            }
            QueryIds::All | QueryIds::Search(_) => {
                Err(QueryError::from("Invalid insert edges query"))
            }
        }
    }
}

impl InsertEdgesQuery {
    fn one_to_many(&self, from: &QueryId) -> Result<Vec<CommandsMut>, QueryError> {
        let mut commands = Vec::<CommandsMut>::new();

        match &self.to {
            QueryIds::Id(to) => commands.push(CommandsMut::InsertEdge(InsertEdge {
                from: from.clone(),
                to: to.clone(),
            })),
            QueryIds::Ids(ids) => {
                for to in ids {
                    commands.push(CommandsMut::InsertEdge(InsertEdge {
                        from: from.clone(),
                        to: to.clone(),
                    }))
                }
            }
            QueryIds::All | QueryIds::Search(_) => {
                return Err(QueryError::from("Invalid insert edges query"))
            }
        }

        Ok(commands)
    }

    fn many_to_many(&self, from: &Vec<QueryId>) -> Result<Vec<CommandsMut>, QueryError> {
        let mut commands = Vec::<CommandsMut>::new();

        match &self.to {
            QueryIds::Id(id) => {
                for from in from {
                    commands.push(CommandsMut::InsertEdge(InsertEdge {
                        from: from.clone(),
                        to: id.clone(),
                    }))
                }
            }
            QueryIds::Ids(ids) => {
                for i in 0..from.len() {
                    commands.push(CommandsMut::InsertEdge(InsertEdge {
                        from: from.get(i).unwrap_or(&QueryId::Id(0)).clone(),
                        to: ids.get(i).unwrap_or(&QueryId::Id(0)).clone(),
                    }))
                }
            }
            QueryIds::All | QueryIds::Search(_) => {
                return Err(QueryError::from("Invalid insert edges query"))
            }
        }

        Ok(commands)
    }

    fn many_to_many_each(&self, from: &Vec<QueryId>) -> Result<Vec<CommandsMut>, QueryError> {
        let mut commands = Vec::<CommandsMut>::new();

        match &self.to {
            QueryIds::Id(to) => {
                for from in from {
                    commands.push(CommandsMut::InsertEdge(InsertEdge {
                        from: from.clone(),
                        to: to.clone(),
                    }))
                }
            }
            QueryIds::Ids(ids) => {
                for from in from {
                    for to in ids {
                        commands.push(CommandsMut::InsertEdge(InsertEdge {
                            from: from.clone(),
                            to: to.clone(),
                        }))
                    }
                }
            }
            QueryIds::All | QueryIds::Search(_) => {
                return Err(QueryError::from("Invalid insert edges query"))
            }
        }

        Ok(commands)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_to_one() {
        let query = InsertEdgesQuery {
            from: QueryIds::Id(QueryId::Id(1)),
            to: QueryIds::Id(QueryId::Id(2)),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![CommandsMut::InsertEdge(InsertEdge {
                from: QueryId::Id(1),
                to: QueryId::Id(2)
            })]
        );
    }

    #[test]
    fn alias_to_alias() {
        let query = InsertEdgesQuery {
            from: QueryIds::Id(QueryId::Alias("alias".to_string())),
            to: QueryIds::Id(QueryId::Alias("alias2".to_string())),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![CommandsMut::InsertEdge(InsertEdge {
                from: QueryId::Alias("alias".to_string()),
                to: QueryId::Alias("alias2".to_string())
            })]
        );
    }

    #[test]
    fn one_to_many() {
        let query = InsertEdgesQuery {
            from: QueryIds::Id(QueryId::Id(1)),
            to: QueryIds::Ids(vec![QueryId::from(2), QueryId::from(3)]),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(1),
                    to: QueryId::Id(2)
                }),
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(1),
                    to: QueryId::Id(3)
                }),
            ]
        );
    }

    #[test]
    fn many_to_one() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(1), QueryId::from(2)]),
            to: QueryIds::Id(QueryId::from(3)),
            values: QueryValues::None,
            each: false,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(1),
                    to: QueryId::Id(3)
                }),
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(2),
                    to: QueryId::Id(3)
                }),
            ]
        );
    }

    #[test]
    fn many_to_each() {
        let query = InsertEdgesQuery {
            from: QueryIds::Ids(vec![QueryId::from(1), QueryId::from(2)]),
            to: QueryIds::Id(QueryId::from(3)),
            values: QueryValues::None,
            each: true,
        };

        assert_eq!(
            query.commands().unwrap(),
            vec![
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(1),
                    to: QueryId::Id(3)
                }),
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(2),
                    to: QueryId::Id(3)
                }),
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
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(1),
                    to: QueryId::Id(3)
                }),
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(2),
                    to: QueryId::Id(4)
                }),
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
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(1),
                    to: QueryId::Id(3)
                }),
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(1),
                    to: QueryId::Id(4)
                }),
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(2),
                    to: QueryId::Id(3)
                }),
                CommandsMut::InsertEdge(InsertEdge {
                    from: QueryId::Id(2),
                    to: QueryId::Id(4)
                }),
            ]
        );
    }

    #[test]
    fn invalid_query_preprocessing_from() {
        let query = InsertEdgesQuery {
            from: QueryIds::All,
            to: QueryIds::Id(QueryId::Id(2)),
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
            from: QueryIds::Id(QueryId::Id(2)),
            to: QueryIds::All,
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
            from: QueryIds::Ids(vec![QueryId::Id(2)]),
            to: QueryIds::All,
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
            from: QueryIds::Ids(vec![QueryId::Id(2)]),
            to: QueryIds::All,
            values: QueryValues::None,
            each: true,
        };

        assert_eq!(
            query.commands().unwrap_err().description,
            QueryError::from("Invalid insert edges query").description
        );
    }
}
