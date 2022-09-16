mod ad32_helper;
mod app_error;
mod chunk_iter;
mod chunk_processor;
mod decode;
mod encode;
mod io_helper;
mod sha3_helper;
mod types;

use app_error::AppError;
use chunk_processor::{
    ChecksumProducer, ChunkProcessor, DeltaExtender, DeltaProducer, IndexedChecksumProducer,
};
use clap::Parser;
use decode::Decoded;
use encode::Encoded;
use io_helper::IOHelper;
use std::path::Path;
use types::{Args, ChecksumStore, SubCommand};

fn produce_signature(
    chunk_size: usize,
    old_file: &Path,
    signature_file: &Path,
) -> Result<(), AppError> {
    let data = old_file.read_from_file()?;

    let chunk_processor = ChunkProcessor::new(chunk_size);

    let checksum_store = chunk_processor.produce_checksum(data)?;

    signature_file.write_to_file(checksum_store.into_encoded()?)
}

pub fn produce_delta(
    chunk_size: usize,
    signature_file: &Path,
    new_file: &Path,
    delta_file: &Path,
) -> Result<(), AppError> {
    let signature_data = signature_file.read_from_file()?;

    let new_file_data = new_file.read_from_file()?;

    let checksum_store = signature_data.decode::<ChecksumStore>()?;

    checksum_store.check_chunk_size_equal(chunk_size)?;

    let checksum_indexed_store = checksum_store.produce_indexed_checksum();

    let delta = checksum_indexed_store.produce_delta(new_file_data)?;

    let full_delta = delta.extend_missed_blocks(&checksum_store.data);

    delta_file.write_to_file(full_delta.into_encoded()?)
}

fn main() -> Result<(), AppError> {
    let args = Args::parse();
    match args.cmd {
        SubCommand::Signature {
            old_file,
            signature_file,
        } => produce_signature(
            args.chunk_size,
            old_file.as_path(),
            signature_file.as_path(),
        ),
        SubCommand::Delta {
            signature_file,
            new_file,
            delta_file,
        } => produce_delta(
            args.chunk_size,
            signature_file.as_path(),
            new_file.as_path(),
            delta_file.as_path(),
        ),
    }
}
