use crate::test_db::test_file::TestFile;
use std::fs::File;
use std::io::Write;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(components(schemas(
    agdb::QueryResult,
    agdb::DbElement,
    agdb::DbId,
    agdb::DbKeyValue,
    agdb::DbKeyOrder,
    agdb::DbValue,
    agdb::DbF64,
    agdb::QueryType,
    agdb::InsertAliasesQuery,
    agdb::InsertEdgesQuery,
    agdb::InsertIndexQuery,
    agdb::InsertNodesQuery,
    agdb::InsertValuesQuery,
    agdb::Comparison,
    agdb::CountComparison,
    agdb::QueryCondition,
    agdb::QueryConditionData,
    agdb::QueryConditionLogic,
    agdb::QueryConditionModifier,
    agdb::QueryId,
    agdb::QueryIds,
    agdb::QueryResult,
    agdb::QueryValues,
    agdb::RemoveAliasesQuery,
    agdb::RemoveIndexQuery,
    agdb::RemoveQuery,
    agdb::RemoveValuesQuery,
    agdb::SearchQuery,
    agdb::SearchQueryAlgorithm,
    agdb::SelectAliasesQuery,
    agdb::SelectAllAliasesQuery,
    agdb::SelectEdgeCountQuery,
    agdb::SelectIndexesQuery,
    agdb::SelectKeyCountQuery,
    agdb::SelectKeysQuery,
    agdb::SelectQuery,
    agdb::SelectValuesQuery,
)))]
pub(crate) struct Api;

#[test]
fn generate_openapi_schema() {
    let _test_file = TestFile::from("test_schema.json");
    let schema = Api::openapi().to_pretty_json().unwrap();
    let mut file = File::create("test_schema.json").unwrap();
    file.write_all(schema.as_bytes()).unwrap();
}
