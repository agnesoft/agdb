use agdb::DbError;

fn main() -> Result<(), DbError> {
    let _db = agdb::Db::new("agdb_server.agdb")?;

    Ok(())
}
