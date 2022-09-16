use std::collections::HashMap;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Debug, Subcommand)]
pub(crate) enum SubCommand {
    Signature {
        #[clap(parse(from_os_str))]
        old_file: std::path::PathBuf,
        signature_file: std::path::PathBuf,
    },
    Delta {
        #[clap(parse(from_os_str))]
        signature_file: std::path::PathBuf,
        #[clap(parse(from_os_str))]
        new_file: std::path::PathBuf,
        delta_file: std::path::PathBuf,
    },
}

/// Represenation of the arguments provided by the user
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long, value_parser, default_value_t = 512)]
    pub(crate) chunk_size: usize,
    #[clap(subcommand)]
    pub(crate) cmd: SubCommand,
}

// Struct to handle weak + strong checksum operations
#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkChecksum {
    pub(crate) ad32: u32,
    pub(crate) hash: [u8; 32],
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DiffBlock {
    pub(crate) start: usize,    // Start of diff position in block
    pub(crate) offset: usize,   // End of diff position in block
    pub(crate) is_mising: bool, // true if Block not found
    pub(crate) buf: Vec<u8>,    // Literal bytes to replace in delta
}

pub type ChecksumStore = Vec<ChunkChecksum>;

pub type IndexedChecksumStore = multimap::MultiMap<u32, ([u8; 32], usize)>;

pub type DeltaStore = HashMap<usize, DiffBlock>;
