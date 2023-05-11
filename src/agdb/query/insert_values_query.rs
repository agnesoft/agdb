use super::query_ids::QueryIds;
use super::query_values::QueryValues;
use crate::commands_mut::insert_value::InsertValue;
use crate::commands_mut::CommandsMut;
use crate::QueryError;
use crate::QueryMut;

pub struct InsertValuesQuery {
    pub ids: QueryIds,
    pub values: QueryValues,
}

impl QueryMut for InsertValuesQuery {
    fn commands(&self) -> Result<Vec<CommandsMut>, QueryError> {
        let mut commands = vec![];

        if let QueryIds::Ids(ids) = &self.ids {
            if let QueryValues::Single(values) = &self.values {
                for id in ids {
                    commands.push(CommandsMut::InsertValue(InsertValue::new(
                        id.clone(),
                        values.clone(),
                    )));
                }
                return Ok(commands);
            } else if let QueryValues::Multi(values) = &self.values {
                if ids.len() != values.len() {
                    return Err(QueryError::from("Ids and values length do not match"));
                }

                for (id, values) in ids.iter().zip(values) {
                    commands.push(CommandsMut::InsertValue(InsertValue::new(
                        id.clone(),
                        values.clone(),
                    )));
                }

                return Ok(commands);
            }
        }

        Err(QueryError::from("Invalid insert aliases query"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::query_id::QueryId;
    use crate::query::search_query::SearchQuery;

    #[test]
    fn values_by_id() {
        let query = InsertValuesQuery {
            ids: QueryIds::Ids(vec![1.into()]),
            values: QueryValues::Ids(QueryIds::Ids(vec![1.into()])),
        };

        assert_eq!(
            query.commands(),
            Err(QueryError::from("Invalid insert aliases query"))
        );
    }

    #[test]
    fn values_by_search() {
        let query = InsertValuesQuery {
            ids: QueryIds::Search(SearchQuery {
                origin: QueryId::from(0),
                destination: QueryId::from(0),
                limit: 0,
                offset: 0,
                order_by: vec![],
                conditions: vec![],
            }),
            values: QueryValues::Single(vec![]),
        };

        assert_eq!(
            query.commands(),
            Err(QueryError::from("Invalid insert aliases query"))
        );
    }

    #[test]
    fn values_none() {
        let query = InsertValuesQuery {
            ids: QueryIds::Ids(vec![1.into()]),
            values: QueryValues::None,
        };

        assert_eq!(
            query.commands(),
            Err(QueryError::from("Invalid insert aliases query"))
        );
    }
}
