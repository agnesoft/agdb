use agdb::DbType;

#[derive(DbType)]
pub(crate) struct BenchUser {
    pub(crate) name: String,
    pub(crate) email: String,
}

#[derive(DbType)]
pub(crate) struct BenchPost {
    pub(crate) title: String,
    pub(crate) body: String,
}

#[derive(DbType)]
pub(crate) struct BenchComment {
    pub(crate) body: String,
}
