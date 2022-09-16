use std::array::TryFromSliceError;

use sha3::{Digest, Keccak256};

pub trait Sha3 {
    fn hash(&self) -> Result<[u8; 32], TryFromSliceError>;
}

impl Sha3 for &[u8] {
    fn hash(&self) -> Result<[u8; 32], TryFromSliceError> {
        let mut hasher = Keccak256::new();
        hasher.update(self);
        hasher.finalize().as_slice().try_into()
    }
}
