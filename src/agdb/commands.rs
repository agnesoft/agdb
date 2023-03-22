pub mod select_id;

use self::select_id::SelectId;

#[derive(Debug, PartialEq)]
pub enum Commands {
    SelectId(SelectId),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::query_id::QueryId;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", Commands::SelectId(SelectId { id: QueryId::Id(0) }));
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            Commands::SelectId(SelectId { id: QueryId::Id(0) }),
            Commands::SelectId(SelectId { id: QueryId::Id(0) })
        );
    }
}
