#[derive(Debug, PartialEq)]
pub struct InsertNode {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", InsertNode {});
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(InsertNode {}, InsertNode {});
    }
}
