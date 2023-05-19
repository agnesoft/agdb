#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Both,
    From,
    To,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derived_from_debug() {
        format!("{:?}", Direction::From);
    }

    #[test]
    fn derived_from_clone() {
        let left = Direction::From;
        let right = left.clone();

        assert_eq!(left, right);
    }

    #[test]
    fn derived_from_partial_eq() {
        assert_eq!(Direction::From, Direction::From);
    }
}
