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

        match &self.ids {
            QueryIds::Ids(ids) => match &self.values {
                QueryValues::None => Ok(commands),
                QueryValues::Ids(_) => Err(QueryError::from("Invalid insert aliases query")),
                QueryValues::Single(values) => {
                    for id in ids {
                        commands.push(CommandsMut::InsertValue(InsertValue::new(
                            id.clone(),
                            values.clone(),
                        )));
                    }
                    Ok(commands)
                }
                QueryValues::Multi(values) => {
                    if ids.len() != values.len() {
                        return Err(QueryError::from("Ids and values length do not match"));
                    }

                    for (id, values) in ids.iter().zip(values) {
                        commands.push(CommandsMut::InsertValue(InsertValue::new(
                            id.clone(),
                            values.clone(),
                        )));
                    }

                    Ok(commands)
                }
            },
            QueryIds::Search(_) => Err(QueryError::from("Invalid insert aliases query")),
        }
    }
}
