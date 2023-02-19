#[derive(Debug, PartialEq)]
pub struct InsertAlias {
    pub alias: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            InsertAlias {
                alias: String::new()
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            InsertAlias {
                alias: String::new()
            },
            InsertAlias {
                alias: String::new()
            }
        );
    }
}
