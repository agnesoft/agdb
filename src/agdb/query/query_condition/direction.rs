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
}
