use agdb::api::ApiDefinition;
use agdb::api::Expression;
use agdb::api::Function;
use agdb::api::NamedType;
use agdb::api::Type;

#[agdb::api_def(insert)]
pub struct QueryBuilder {}

#[agdb::api_def(aliases)]
pub struct Insert {}

#[agdb::api_def()]
pub struct QueryAliases(pub Vec<String>);

#[agdb::api_def()]
pub struct InsertAliases(pub QueryAliases);

impl QueryBuilder {
    pub fn __insert() -> Function {
        Function {
            name: "insert",
            args: vec![],
            expressions: vec![Expression::create_return(Insert::def)],
            ret: Insert::def,
        }
    }
}

impl Insert {
    pub fn __aliases() -> Function {
        Function {
            name: "aliases",
            args: vec![
                NamedType {
                    name: "self",
                    ty: || Type::None,
                },
                NamedType {
                    name: "names",
                    ty: QueryAliases::def,
                },
            ],
            expressions: vec![Expression::create_return_arg_t(InsertAliases::def, "names")],
            ret: InsertAliases::def,
        }
    }
}
