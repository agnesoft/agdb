#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64() {
        let number = -10_i64;
        let bytes = number.serialize();
        let actual = i64::deserialize(bytes);

        assert_eq!(actual, number)
    }

    #[test]
    fn u64() {
        let number = 10_u64;
        let bytes = number.serialize();
        let actual = u64::deserialize(bytes);

        assert_eq!(actual, number)
    }
}
