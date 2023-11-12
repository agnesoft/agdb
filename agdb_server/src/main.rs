use agdb::DbError;

fn main() -> Result<(), DbError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_test() {
        assert_eq!(main(), Ok(()))
    }
}
