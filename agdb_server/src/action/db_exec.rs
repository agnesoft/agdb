use super::DbPool;
use super::ServerDb;
use crate::action::Action;
use crate::action::ClusterActionResult;
use crate::server_error::ServerResult;
use agdb::AgdbDeSerialize;
use agdb::DbUserValue;
use agdb::QueryResult;
use agdb_api::Queries;
use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, AgdbDeSerialize)]
pub(crate) struct DbExec {
    pub(crate) user: String,
    pub(crate) owner: String,
    pub(crate) db: String,
    pub(crate) queries: Queries,
}

impl Action for DbExec {
    async fn exec(self, _db: ServerDb, db_pool: DbPool) -> ServerResult<ClusterActionResult> {
        Ok(ClusterActionResult::QueryResults(
            db_pool
                .exec_mut(&self.owner, &self.db, &self.user, self.queries)
                .await?,
        ))
    }
}

impl DbUserValue for DbExec {
    type ValueType = DbExec;

    fn db_id(&self) -> Option<agdb::QueryId> {
        None
    }

    fn db_keys() -> Vec<agdb::DbValue> {
        vec!["user".into(), "owner".into(), "db".into(), "queries".into()]
    }

    fn from_db_element(element: &agdb::DbElement) -> Result<Self::ValueType, agdb::DbError> {
        Ok(Self {
            user: element.values[0].value.string()?.clone(),
            owner: element.values[1].value.string()?.clone(),
            db: element.values[2].value.string()?.clone(),
            queries: serde_json::from_str(element.values[3].value.string()?)
                .map_err(|e| agdb::DbError::from(e.to_string()))?,
        })
    }

    fn to_db_values(&self) -> Vec<agdb::DbKeyValue> {
        let queries = serde_json::to_string(&self.queries).unwrap_or_default();

        vec![
            ("user", &self.user).into(),
            ("owner", &self.owner).into(),
            ("db", &self.db).into(),
            ("queries", queries).into(),
        ]
    }
}

impl TryFrom<&agdb::DbElement> for DbExec {
    type Error = agdb::DbError;

    #[track_caller]
    fn try_from(value: &agdb::DbElement) -> std::result::Result<Self, Self::Error> {
        DbExec::from_db_element(value)
    }
}

impl TryFrom<QueryResult> for DbExec {
    type Error = agdb::DbError;

    #[track_caller]
    fn try_from(value: QueryResult) -> Result<Self, Self::Error> {
        value
            .elements
            .first()
            .ok_or(Self::Error::from("No element found"))?
            .try_into()
    }
}
