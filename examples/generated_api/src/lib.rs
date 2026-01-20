use agdb::DbType;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::borrow::Borrow;

type AgdbApiResult<T> = Result<T, AgdbApiError>;

pub trait AgdbApiClient {}

pub trait HttpClient {
    fn delete(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<u16>> + Send;
    fn get<T: DeserializeOwned + Send>(
        &self,
        uri: &str,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<(u16, T)>> + Send;
    fn post<T: Serialize + Send, R: DeserializeOwned + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<(u16, R)>> + Send;
    fn put<T: Serialize + Send>(
        &self,
        uri: &str,
        json: Option<T>,
        token: &Option<String>,
    ) -> impl std::future::Future<Output = AgdbApiResult<u16>> + Send;
}

pub trait SearchQueryBuilder: agdb::api_def::TypeDefinition {
    fn search_mut(&mut self) -> &mut SearchQuery;
}

enum SearchQueryAlgorithm {
    BreadthFirst,
    DepthFirst,
    Index,
    Elements,
}

struct DbId(i64);

enum QueryId {
    Id(i64),
    Alias(String),
}

struct SearchQuery {
    algorithm: SearchQueryAlgorithm,
    origin: QueryId,
    destination: QueryId,
    limit: u64,
    offset: u64,
    order_by: Vec<DbKeyOrder>,
    conditions: Vec<QueryCondition>,
}

enum QueryIds {
    Ids(Vec<QueryId>),
    Search {
        algorithm: SearchQueryAlgorithm,
        origin: QueryId,
        destination: QueryId,
        limit: u64,
        offset: u64,
        order_by: Vec<DbKeyOrder>,
        conditions: Vec<QueryCondition>,
    },
}

struct InsertAliasesQuery {
    ids: QueryIds,
    aliases: Vec<String>,
}

struct InsertAliasesIds(InsertAliasesQuery);

struct InsertAliases(InsertAliasesQuery);

enum QueryValues {
    Single(Vec<DbKeyValue>),
    Multi(Vec<Vec<DbKeyValue>>),
}

struct InsertEdgesQuery {
    from: QueryIds,
    to: QueryIds,
    ids: QueryIds,
    values: QueryValues,
    each: bool,
}

struct InsertEdgesValues(InsertEdgesQuery);

struct InsertEdgesEach(InsertEdgesQuery);

struct InsertEdgesFromTo(InsertEdgesQuery);

struct InsertEdgesFrom(InsertEdgesQuery);

struct InsertEdgesIds(InsertEdgesQuery);

struct InsertEdges(InsertEdgesQuery);

struct InsertValuesQuery {
    ids: QueryIds,
    values: QueryValues,
}

struct InsertValuesIds(InsertValuesQuery);

struct DbF64(f64);

enum DbValue {
    Bytes(Vec<u8>),
    I64(i64),
    U64(u64),
    F64(f64),
    String(String),
    VecI64(Vec<i64>),
    VecU64(Vec<u64>),
    VecF64(Vec<DbF64>),
    VecString(Vec<String>),
}

struct InsertIndexQuery(DbValue);

struct InsertIndex(DbValue);

struct InsertNodesQuery {
    count: u64,
    values: QueryValues,
    aliases: Vec<String>,
    ids: QueryIds,
}

struct InsertNodesValues(InsertNodesQuery);

struct InsertNodesAliases(InsertNodesQuery);

struct InsertNodesCount(InsertNodesQuery);

struct InsertNodesIds(InsertNodesQuery);

struct InsertNodes(InsertNodesQuery);

struct Search<T: SearchQueryBuilder>(T);

struct InsertValues(InsertValuesQuery);

struct Insert {}
impl Insert {
    pub fn aliases<T: Into<QueryAliases>>(self, names: T) -> InsertAliases {
        todo!()
    }
    pub fn edges(self) -> InsertEdges {
        todo!()
    }
    pub fn element<T: DbType + Borrow<T>>(self, elem: T) -> InsertValuesIds {
        todo!()
    }
    pub fn elements<T: DbType>(self, elems: &[T]) -> InsertValuesIds {
        todo!()
    }
    pub fn index<T: Into<DbValue>>(self, key: T) -> InsertIndex {
        todo!()
    }
    pub fn nodes(self) -> InsertNodes {
        todo!()
    }
    pub fn values<T: Into<MultiValues>>(self, key_values: T) -> InsertValues {
        todo!()
    }
    pub fn values_uniform<T: Into<SingleValues>>(self, key_values: T) -> InsertValues {
        todo!()
    }
}

struct RemoveAliasesQuery(Vec<String>);

struct RemoveAliases(RemoveAliasesQuery);

struct RemoveQuery(QueryIds);

struct RemoveIds(RemoveQuery);

struct RemoveIndexQuery(DbValue);

struct RemoveIndex(DbValue);

struct SelectValuesQuery {
    keys: Vec<DbValue>,
    ids: QueryIds,
}

struct RemoveValuesQuery(SelectValuesQuery);

struct RemoveValuesIds(RemoveValuesQuery);

struct RemoveValues(RemoveValuesQuery);

struct Remove {}
impl Remove {
    pub fn aliases<T: Into<QueryAliases>>(self, names: T) -> RemoveAliases {
        todo!()
    }
    pub fn ids<T: Into<QueryIds>>(self, ids: T) -> RemoveIds {
        todo!()
    }
    pub fn index<T: Into<DbValue>>(self, key: T) -> RemoveIndex {
        todo!()
    }
    pub fn search(self) -> Search<T> {
        todo!()
    }
    pub fn values<T: Into<DbValues>>(self, keys: T) -> RemoveValues {
        todo!()
    }
}

struct SelectAliasesQuery(QueryIds);

struct SelectAliasesIds(SelectAliasesQuery);

struct SelectAllAliasesQuery {}

struct SelectAliases(SelectAliasesQuery);

struct SelectEdgeCountQuery {
    ids: QueryIds,
    from: bool,
    to: bool,
}

struct SelectEdgeCountIds(SelectEdgeCountQuery);

struct SelectEdgeCount(SelectEdgeCountQuery);

struct SelectValuesIds(SelectValuesQuery);

struct SelectValues {
    query: SelectValuesQuery,
    element_id: Option<DbValue>,
    limit: u64,
}
impl SelectValues {
    pub fn ids<T: Into<QueryIds>>(mut self, ids: T) -> SelectValuesIds {
        todo!()
    }
    pub fn search(mut self) -> Search<T> {
        todo!()
    }
}

struct SelectIds(SelectValuesQuery);

struct SelectIndexesQuery {}

struct SelectIndexes {}
impl SelectIndexes {
    pub fn query(&self) -> SelectIndexesQuery {
        todo!()
    }
}

struct SelectKeysQuery(QueryIds);

struct SelectKeysIds(SelectKeysQuery);

struct SelectKeys(SelectKeysQuery);

struct SelectKeyCountQuery(QueryIds);

struct SelectKeyCountIds(SelectKeyCountQuery);

struct SelectKeyCount(SelectKeyCountQuery);

struct SelectNodeCountQuery {}

struct SelectNodeCount {}
impl SelectNodeCount {
    pub fn query(self) -> SelectNodeCountQuery {
        todo!()
    }
}

struct Select {}
impl Select {
    pub fn aliases(self) -> SelectAliases {
        todo!()
    }
    pub fn edge_count(self) -> SelectEdgeCount {
        todo!()
    }
    pub fn edge_count_from(self) -> SelectEdgeCount {
        todo!()
    }
    pub fn edge_count_to(self) -> SelectEdgeCount {
        todo!()
    }
    pub fn element<T: DbType>(self) -> SelectValues {
        todo!()
    }
    pub fn elements<T: DbType>(self) -> SelectValues {
        todo!()
    }
    pub fn ids<T: Into<QueryIds>>(self, ids: T) -> SelectIds {
        todo!()
    }
    pub fn indexes(self) -> SelectIndexes {
        todo!()
    }
    pub fn keys(self) -> SelectKeys {
        todo!()
    }
    pub fn key_count(self) -> SelectKeyCount {
        todo!()
    }
    pub fn node_count(self) -> SelectNodeCount {
        todo!()
    }
    pub fn search(self) -> Search<T> {
        todo!()
    }
    pub fn values<T: Into<DbValues>>(self, keys: T) -> SelectValues {
        todo!()
    }
}

struct QueryBuilder {}
impl QueryBuilder {
    pub fn insert() -> Insert {
        todo!()
    }
    pub fn remove() -> Remove {
        todo!()
    }
    pub fn search() -> Search<T> {
        todo!()
    }
    pub fn select() -> Select {
        todo!()
    }
}

enum DbKind {
    Memory,
    Mapped,
    File,
}

enum DbResource {
    All,
    Db,
    Audit,
    Backup,
}

enum DbUserRole {
    Admin,
    Write,
    Read,
}

struct AgdbApi<T: AgdbApiClient> {
    client: T,
    address: String,
    base_url: String,
    token: Option<String>,
}
impl<T: AgdbApiClient> AgdbApi<T> {
    pub fn new(client: T, address: &str) -> Self {
        todo!()
    }
    pub fn address(&self) -> &str {
        todo!()
    }
    pub fn base_url(&self) -> &str {
        todo!()
    }
    pub async fn admin_db_add(
        &self,
        owner: &str,
        db: &str,
        db_type: DbKind,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_audit(
        &self,
        owner: &str,
        db: &str,
    ) -> Result<(u16, DbAudit), AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_backup(&self, owner: &str, db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_clear(
        &self,
        owner: &str,
        db: &str,
        resource: DbResource,
    ) -> Result<(u16, ServerDatabase), AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_convert(
        &self,
        owner: &str,
        db: &str,
        db_type: DbKind,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_copy(
        &self,
        owner: &str,
        db: &str,
        new_owner: &str,
        new_db: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_delete(&self, owner: &str, db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_exec(
        &self,
        owner: &str,
        db: &str,
        queries: &[QueryType],
    ) -> Result<(u16, Vec<QueryResult>), AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_exec_mut(
        &self,
        owner: &str,
        db: &str,
        queries: &[QueryType],
    ) -> Result<(u16, Vec<QueryResult>), AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_list(&self) -> Result<(u16, Vec<ServerDatabase>), AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_optimize(
        &self,
        owner: &str,
        db: &str,
    ) -> Result<(u16, ServerDatabase), AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_remove(&self, owner: &str, db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_rename(
        &self,
        owner: &str,
        db: &str,
        new_owner: &str,
        new_db: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_restore(&self, owner: &str, db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_user_add(
        &self,
        owner: &str,
        db: &str,
        username: &str,
        db_role: DbUserRole,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_user_list(
        &self,
        owner: &str,
        db: &str,
    ) -> Result<(u16, Vec<DbUser>), AgdbApiError> {
        todo!()
    }
    pub async fn admin_db_user_remove(
        &self,
        owner: &str,
        db: &str,
        username: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_shutdown(&self) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_status(&self) -> Result<(u16, AdminStatus), AgdbApiError> {
        todo!()
    }
    pub async fn admin_user_add(
        &self,
        username: &str,
        password: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_user_change_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_user_list(&self) -> Result<(u16, Vec<UserStatus>), AgdbApiError> {
        todo!()
    }
    pub async fn admin_user_logout(&self, username: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_user_logout_all(&self) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn admin_user_delete(&self, username: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_add(
        &self,
        owner: &str,
        db: &str,
        db_type: DbKind,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn cluster_admin_user_logout(&self, username: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn cluster_admin_user_logout_all(&self) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn cluster_user_login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn cluster_user_logout(&mut self) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn cluster_status(&self) -> Result<(u16, Vec<ClusterStatus>), AgdbApiError> {
        todo!()
    }
    pub async fn db_audit(&self, owner: &str, db: &str) -> Result<(u16, DbAudit), AgdbApiError> {
        todo!()
    }
    pub async fn db_backup(&self, owner: &str, db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_clear(
        &self,
        owner: &str,
        db: &str,
        resource: DbResource,
    ) -> Result<(u16, ServerDatabase), AgdbApiError> {
        todo!()
    }
    pub async fn db_convert(
        &self,
        owner: &str,
        db: &str,
        db_type: DbKind,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_copy(&self, owner: &str, db: &str, new_db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_delete(&self, owner: &str, db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_exec(
        &self,
        owner: &str,
        db: &str,
        queries: &[QueryType],
    ) -> Result<(u16, Vec<QueryResult>), AgdbApiError> {
        todo!()
    }
    pub async fn db_exec_mut(
        &self,
        owner: &str,
        db: &str,
        queries: &[QueryType],
    ) -> Result<(u16, Vec<QueryResult>), AgdbApiError> {
        todo!()
    }
    pub async fn db_list(&self) -> Result<(u16, Vec<ServerDatabase>), AgdbApiError> {
        todo!()
    }
    pub async fn db_optimize(
        &self,
        owner: &str,
        db: &str,
    ) -> Result<(u16, ServerDatabase), AgdbApiError> {
        todo!()
    }
    pub async fn db_remove(&self, owner: &str, db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_rename(
        &self,
        owner: &str,
        db: &str,
        new_db: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_restore(&self, owner: &str, db: &str) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_user_add(
        &self,
        owner: &str,
        db: &str,
        username: &str,
        db_role: DbUserRole,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn db_user_list(
        &self,
        owner: &str,
        db: &str,
    ) -> Result<(u16, Vec<DbUser>), AgdbApiError> {
        todo!()
    }
    pub async fn db_user_remove(
        &self,
        owner: &str,
        db: &str,
        username: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn status(&self) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn user_login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn user_logout(&mut self) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn user_change_password(
        &self,
        old_password: &str,
        new_pasword: &str,
    ) -> Result<u16, AgdbApiError> {
        todo!()
    }
    pub async fn user_status(&self) -> Result<(u16, UserStatus), AgdbApiError> {
        todo!()
    }
    pub fn url(&self, uri: &str) -> String {
        todo!()
    }
}

struct AgdbApiError {
    status: u16,
    description: String,
}

enum DbKeyOrder {
    Asc(DbValue),
    Desc(DbValue),
}

struct DbKeyValue {
    key: DbValue,
    value: DbValue,
}

enum QueryConditionLogic {
    And,
    Or,
}

enum QueryConditionModifier {
    None,
    Beyond,
    Not,
    NotBeyond,
}

enum CountComparison {
    Equal(u64),
    GreaterThan(u64),
    GreaterThanOrEqual(u64),
    LessThan(u64),
    LessThanOrEqual(u64),
    NotEqual(u64),
}

enum Comparison {
    Equal(DbValue),
    GreaterThan(DbValue),
    GreaterThanOrEqual(DbValue),
    LessThan(DbValue),
    LessThanOrEqual(DbValue),
    NotEqual(DbValue),
    Contains(DbValue),
    StartsWith(DbValue),
    EndsWith(DbValue),
}

struct KeyValueComparison {
    key: DbValue,
    value: Comparison,
}

enum QueryConditionData {
    Distance(CountComparison),
    Edge,
    EdgeCount(CountComparison),
    EdgeCountFrom(CountComparison),
    EdgeCountTo(CountComparison),
    Ids(Vec<QueryId>),
    KeyValue { key: DbValue, value: Comparison },
    Keys(Vec<DbValue>),
    Node,
    Where(Vec<QueryCondition>),
}

struct QueryCondition {
    logic: QueryConditionLogic,
    modifier: QueryConditionModifier,
    data: QueryConditionData,
}

struct QueryAliases(Vec<String>);

struct MultiValues(Vec<Vec<DbKeyValue>>);

struct SingleValues(Vec<DbKeyValue>);

struct DbValues(Vec<DbValue>);

struct ServerDatabase {
    db: String,
    owner: String,
    db_type: DbKind,
    role: DbUserRole,
    size: u64,
    backup: u64,
}

enum QueryType {
    InsertAlias {
        ids: QueryIds,
        aliases: Vec<String>,
    },
    InsertEdges {
        from: QueryIds,
        to: QueryIds,
        ids: QueryIds,
        values: QueryValues,
        each: bool,
    },
    InsertIndex(DbValue),
    InsertNodes {
        count: u64,
        values: QueryValues,
        aliases: Vec<String>,
        ids: QueryIds,
    },
    InsertValues {
        ids: QueryIds,
        values: QueryValues,
    },
    Remove(QueryIds),
    RemoveAliases(Vec<String>),
    RemoveIndex(DbValue),
    RemoveValues(SelectValuesQuery),
    Search {
        algorithm: SearchQueryAlgorithm,
        origin: QueryId,
        destination: QueryId,
        limit: u64,
        offset: u64,
        order_by: Vec<DbKeyOrder>,
        conditions: Vec<QueryCondition>,
    },
    SelectAliases(QueryIds),
    SelectAllAliases {},
    SelectEdgeCount {
        ids: QueryIds,
        from: bool,
        to: bool,
    },
    SelectIndexes {},
    SelectKeys(QueryIds),
    SelectKeyCount(QueryIds),
    SelectNodeCount {},
    SelectValues {
        keys: Vec<DbValue>,
        ids: QueryIds,
    },
}

struct QueryResult {
    result: i64,
    elements: Vec<DbElement>,
}

struct AdminStatus {
    uptime: u64,
    dbs: u64,
    users: u64,
    logged_in_users: u64,
    size: u64,
}

struct UserStatus {
    username: String,
    login: bool,
    admin: bool,
}

struct ClusterStatus {
    address: String,
    status: bool,
    leader: bool,
}

struct DbAudit(Vec<QueryAudit>);

struct DbUser {
    username: String,
    role: DbUserRole,
}
