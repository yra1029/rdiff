use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use crate::app_error::AppError;

pub trait IOHelper {
    fn read_from_file(&self) -> Result<Vec<u8>, AppError>;
    fn write_to_file(&self, buf: Vec<u8>) -> Result<(), AppError>;
}

pub(crate) fn file_size<T>(path: T) -> Result<usize, AppError>
where
    T: AsRef<Path>,
{
    let metadata = fs::metadata(path)?;

    Ok(metadata.len() as usize)
}

impl<T> IOHelper for T
where
    T: AsRef<Path>,
{
    fn read_from_file(&self) -> Result<Vec<u8>, AppError> {
        let file_size = file_size(self)?;

        let mut f = File::open(self)?;

        let mut buffer = vec![0 as u8; file_size];

        let n = f.read_to_end(&mut buffer)?;

        match n {
            _ if file_size == n => Ok(buffer),
            _ => Err(AppError::IOError(String::from(
                "Could not read all data from file",
            ))),
        }
    }

    fn write_to_file(&self, buf: Vec<u8>) -> Result<(), AppError> {
        let mut file = OpenOptions::new().write(true).open(self)?;

        file.write_all(&buf).map_err(AppError::from)
    }
}
