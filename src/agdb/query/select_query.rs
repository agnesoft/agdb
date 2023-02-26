use super::query_ids::QueryIds;
use crate::commands::{select_id::SelectId, Commands};

pub struct SelectQuery(pub QueryIds);

impl SelectQuery {
    pub fn commands(&self) -> Vec<Commands> {
        match &self.0 {
            QueryIds::All | QueryIds::Search(_) => panic!("Invalid query"),
            QueryIds::Id(id) => vec![Commands::SelectId(SelectId { id: id.clone() })],
            QueryIds::Ids(ids) => ids
                .iter()
                .map(|id| Commands::SelectId(SelectId { id: id.clone() }))
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utilities::catch_unwind_silent::catch_unwind_silent;

    #[test]
    fn invalid_query() {
        let result = catch_unwind_silent(|| {
            let query = SelectQuery(QueryIds::All);

            query.commands();
        });

        assert_eq!(
            *result.unwrap_err().downcast_ref::<&str>().unwrap(),
            "Invalid query"
        );
    }
}
