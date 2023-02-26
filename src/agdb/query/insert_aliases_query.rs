use super::query_id::QueryId;
use super::query_ids::QueryIds;
use crate::commands::insert_alias::InsertAlias;
use crate::commands::Commands;

pub struct InsertAliasesQuery {
    pub ids: QueryIds,
    pub aliases: Vec<String>,
}

impl InsertAliasesQuery {
    pub(crate) fn commands(&self) -> Vec<Commands> {
        match &self.ids {
            QueryIds::All | QueryIds::Search(_) => panic!("Invalid query"),
            QueryIds::Id(id) => self.id(id),
            QueryIds::Ids(ids) => self.ids(&ids),
        }
    }

    fn id(&self, id: &super::query_id::QueryId) -> Vec<Commands> {
        vec![Commands::InsertAlias(InsertAlias {
            id: Some(id.clone()),
            alias: self.aliases[0].clone(),
        })]
    }

    fn ids(&self, ids: &Vec<QueryId>) -> Vec<Commands> {
        let mut commands = Vec::<Commands>::new();

        for (id, alias) in ids.iter().zip(self.aliases.iter()) {
            commands.push(Commands::InsertAlias(InsertAlias {
                id: Some(id.clone()),
                alias: alias.clone(),
            }));
        }

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::catch_unwind_silent::catch_unwind_silent;

    #[test]
    fn invalid_query() {
        let result = catch_unwind_silent(|| {
            let query = InsertAliasesQuery {
                ids: QueryIds::All,
                aliases: vec![],
            };

            query.commands();
        });

        assert_eq!(
            *result.unwrap_err().downcast_ref::<&str>().unwrap(),
            "Invalid query"
        );
    }
}
