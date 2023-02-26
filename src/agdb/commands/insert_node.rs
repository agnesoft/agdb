#[derive(Debug, PartialEq)]
pub struct InsertNode {
    pub alias: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertNode { alias: None });
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(InsertNode { alias: None }, InsertNode { alias: None });
    }
}
