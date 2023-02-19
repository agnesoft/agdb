#[derive(Debug, PartialEq)]
pub struct RemoveEdge {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", RemoveEdge {});
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(RemoveEdge {}, RemoveEdge {});
    }
}
