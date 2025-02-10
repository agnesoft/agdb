use agdb_derive::api_def;

use crate::api::ApiDefinition;
use crate::api::Expression;
use crate::api::Function;
use crate::api::NamedType;
use crate::api::Type;
use crate::InsertAliasesQuery;
use crate::QueryIds;

#[api_def(insert)]
pub struct QueryBuilder {}

#[api_def(aliases)]
pub struct Insert {}

#[expect(dead_code)]
#[api_def(ids)]
pub struct InsertAliases(InsertAliasesQuery);

#[expect(dead_code)]
#[api_def(query)]
pub struct InsertAliasesIds(InsertAliasesQuery);

fn __insert_def() -> Function {
    Function {
        name: "insert",
        args: vec![],
        expressions: vec![Expression::create_return(Insert::def)],
        ret: QueryBuilder::def,
    }
}

fn __aliases_def() -> Function {
    Function {
        name: "aliases",
        args: vec![NamedType {
            name: "aliases",
            ty: Vec::<String>::def,
        }],
        expressions: vec![
            Expression::create("q", InsertAliases::def),
            Expression::assign_fields("q", vec![".", "aliases"], "aliases"),
            Expression::ret("q"),
        ],
        ret: QueryBuilder::def,
    }
}

fn __ids_def() -> Function {
    Function {
        name: "ids",
        args: vec![NamedType {
            name: "ids",
            ty: QueryIds::def,
        }],
        expressions: vec![
            Expression::create_arg("q", InsertAliasesIds::def, "."),
            Expression::assign_fields("q", vec![".", "ids"], "ids"),
            Expression::ret("q"),
        ],
        ret: InsertAliasesIds::def,
    }
}

fn __query_def() -> Function {
    Function {
        name: "query",
        args: vec![],
        expressions: vec![Expression::ret(".")],
        ret: InsertAliasesQuery::def,
    }
}
