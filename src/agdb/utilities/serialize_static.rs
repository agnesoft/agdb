use std::mem::size_of;

pub trait SerializeStatic: Sized {
    fn static_serialized_size() -> u64 {
        size_of::<Self>() as u64
    }
}

impl SerializeStatic for i64 {}
impl SerializeStatic for u64 {}
impl SerializeStatic for f64 {}

impl SerializeStatic for usize {
    fn static_serialized_size() -> u64 {
        size_of::<u64>() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usize() {
        assert_eq!(
            usize::static_serialized_size(),
            u64::static_serialized_size()
        );
    }
}
