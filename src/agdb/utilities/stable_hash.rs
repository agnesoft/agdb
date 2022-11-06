use std::{mem::size_of, ops::BitXor};

const HASH_CONSTANT: u64 = 0x517cc1b727220a95;

pub trait StableHash {
    fn stable_hash(&self) -> u64;

    fn add_to_hash(hash: &mut u64, value: u64) {
        *hash = hash
            .rotate_left(5)
            .bitxor(value)
            .wrapping_mul(HASH_CONSTANT);
    }
}

impl StableHash for i64 {
    fn stable_hash(&self) -> u64 {
        *self as u64
    }
}

impl StableHash for u64 {
    fn stable_hash(&self) -> u64 {
        *self
    }
}

impl StableHash for String {
    fn stable_hash(&self) -> u64 {
        self.as_bytes().stable_hash()
    }
}

impl StableHash for &[u8] {
    fn stable_hash(&self) -> u64 {
        const CHUNK_SIZE: usize = size_of::<u64>();
        let chunks = self.len() / CHUNK_SIZE;
        let remainder = self.len() % CHUNK_SIZE;
        let mut hash = 0_u64;

        for chunk in 0..chunks {
            let begin = chunk * CHUNK_SIZE;
            let end = begin + CHUNK_SIZE;
            Self::add_to_hash(
                &mut hash,
                u64::from_le_bytes(self[begin..end].try_into().unwrap()),
            );
        }

        if remainder != 0 {
            let begin = chunks * CHUNK_SIZE;
            let end = begin + remainder;
            let mut data = [0_u8; CHUNK_SIZE];
            data[0..remainder].copy_from_slice(&self[begin..end]);
            Self::add_to_hash(&mut hash, u64::from_le_bytes(data));
        }

        Self::add_to_hash(&mut hash, self.len() as u64);

        hash
    }
}

impl StableHash for Vec<u8> {
    fn stable_hash(&self) -> u64 {
        self.as_slice().stable_hash()
    }
}

impl<T: StableHash> StableHash for Vec<T> {
    fn stable_hash(&self) -> u64 {
        let mut hash = 0_u64;

        for value in self {
            Self::add_to_hash(&mut hash, value.stable_hash());
        }

        Self::add_to_hash(&mut hash, self.len() as u64);

        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i64() {
        assert_eq!(10_i64.stable_hash(), 10_u64);
    }

    #[test]
    fn u64() {
        assert_eq!(10_u64.stable_hash(), 10_u64);
    }

    #[test]
    fn string() {
        assert_eq!("".to_string().stable_hash(), 0);

        let string_hash = "Hello, World!".to_string().stable_hash();
        let other_string_hash = "Hello".to_string().stable_hash();

        assert_ne!(string_hash, 0);
        assert_ne!(" ".to_string().stable_hash(), 0);
        assert_ne!(string_hash, other_string_hash);
        assert_eq!(string_hash, "Hello, World!".to_string().stable_hash());
    }

    #[test]
    fn vec_u8() {
        let vec_hash = vec![1_u8, 2_u8, 3_u8].stable_hash();
        let other_vec_hash = vec![3_u8, 2_u8, 1_u8].stable_hash();

        assert_ne!(vec_hash, 0);
        assert_ne!(vec![0_u8].stable_hash(), 0);
        assert_ne!(vec_hash, other_vec_hash);
        assert_eq!(vec_hash, vec![1_u8, 2_u8, 3_u8].stable_hash());
    }

    #[test]
    fn vec_64() {
        let vec_hash = vec![1_u64, 2_u64, 3_u64].stable_hash();
        let other_vec_hash = vec![1_i64, 2_i64, 3_i64].stable_hash();

        assert_ne!(vec_hash, 0);
        assert_ne!(vec![0_u64].stable_hash(), 0);
        assert_eq!(vec_hash, other_vec_hash);
        assert_eq!(vec_hash, vec![1_u64, 2_u64, 3_u64].stable_hash());
    }
}
