use std::{array::TryFromSliceError, fmt, io::Error};

#[derive(Debug)]
pub enum AppError {
    IOError(String),
    TryFromSliceError(String),
    FileTooShort,
    SerializeError(String),
    IncompatibleChunkSize(String),
    IncompatibleDataSize(String),
    IndexCorrupted,
}

const FILE_TOO_SHORT_DESCRIPTION: &str =
    "File to small for processing, please provide bigger one or descrease the block size";

const INDEX_CORRUPTED_DESRIPTION: &str = "Something went wrong please or signature file corrupted";

impl std::error::Error for AppError {
    fn description(&self) -> &str {
        match self {
            AppError::IOError(err_data) => &err_data,
            AppError::TryFromSliceError(err_data) => &err_data,
            AppError::FileTooShort => FILE_TOO_SHORT_DESCRIPTION,
            AppError::SerializeError(err_data) => &err_data,
            AppError::IncompatibleChunkSize(err_data) => &err_data,
            AppError::IncompatibleDataSize(err_data) => &err_data,
            AppError::IndexCorrupted => INDEX_CORRUPTED_DESRIPTION,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppError::IOError(err_data) => f.write_str(&err_data),
            AppError::TryFromSliceError(err_data) => f.write_str(&err_data),
            AppError::FileTooShort => f.write_str(FILE_TOO_SHORT_DESCRIPTION),
            AppError::SerializeError(err_data) => f.write_str(&err_data),
            AppError::IncompatibleChunkSize(err_data) => f.write_str(&err_data),
            AppError::IncompatibleDataSize(err_data) => f.write_str(&err_data),
            AppError::IndexCorrupted => f.write_str(INDEX_CORRUPTED_DESRIPTION),
        }
    }
}

impl From<Error> for AppError {
    fn from(error: Error) -> Self {
        AppError::IOError(format!("AppError::IOError occured: {}", error))
    }
}

impl From<TryFromSliceError> for AppError {
    fn from(error: TryFromSliceError) -> Self {
        AppError::TryFromSliceError(format!("AppError::TryFromSliceError occured: {}", error))
    }
}

impl From<Box<bincode::ErrorKind>> for AppError {
    fn from(error: Box<bincode::ErrorKind>) -> Self {
        AppError::SerializeError(format!("AppError::SerializeError occured: {}", error))
    }
}
