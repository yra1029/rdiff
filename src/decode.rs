use serde::{Deserialize, Serialize};

use crate::{app_error::AppError, chunk_processor::ChunkProcessor};

pub trait Decoded {
    fn decode<'a, T: Serialize + Deserialize<'a>>(&'a self) -> Result<ChunkProcessor<T>, AppError>;
}

impl Decoded for Vec<u8> {
    fn decode<'a, T: Serialize + Deserialize<'a>>(&'a self) -> Result<ChunkProcessor<T>, AppError> {
        bincode::deserialize::<'a, ChunkProcessor<T>>(&self[..]).map_err(AppError::from)
    }
}
