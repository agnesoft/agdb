#[derive(Debug, PartialEq)]
pub struct RemoveAlias {
    pub alias: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!(
            "{:?}",
            RemoveAlias {
                alias: String::new()
            }
        );
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(
            RemoveAlias {
                alias: String::new()
            },
            RemoveAlias {
                alias: String::new()
            }
        );
    }
}
