use serde::{Deserialize, Serialize};

use crate::app_error::AppError;

pub trait Encoded {
    fn into_encoded(&self) -> Result<Vec<u8>, AppError>;
}

impl<'a, T: Serialize + Deserialize<'a>> Encoded for T
where
    T: Serialize,
{
    fn into_encoded(&self) -> Result<Vec<u8>, AppError> {
        bincode::serialize(self).map_err(AppError::from)
    }
}
