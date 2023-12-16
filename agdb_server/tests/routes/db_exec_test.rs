use crate::TestServer;
use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
enum Query {
    AbortTransaction,
    BeginTransaction,
    CommitTransaction,
    InsertNodes(agdb::InsertNodesQuery),
    InsertEdges(agdb::InsertEdgesQuery),
    InsertAliases(agdb::InsertAliasesQuery),
    InsertValues(agdb::InsertValuesQuery),
    Remove(agdb::RemoveQuery),
    RemoveAliases(agdb::RemoveAliasesQuery),
    RemoveValues(agdb::RemoveValuesQuery),
    Select(agdb::SelectQuery),
    SelectValues(agdb::SelectValuesQuery),
    SelectKeys(agdb::SelectKeysQuery),
    SelectKeyCount(agdb::SelectKeyCountQuery),
    SelectAliases(agdb::SelectAliasesQuery),
    SelectAllAliases(agdb::SelectAllAliasesQuery),
    Search(agdb::SearchQuery),
}

#[tokio::test]
async fn exec() -> anyhow::Result<()> {
    let server = TestServer::new().await?;
    let user = server.init_user().await?;
    let db = server.init_db("memory", &user).await?;

    Ok(())
}
