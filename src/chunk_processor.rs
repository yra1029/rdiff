use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    ad32_helper::Ad32,
    app_error::AppError,
    chunk_iter::{ChunkIter, SkipChunk},
    sha3_helper::Sha3,
    types::{ChecksumStore, ChunkChecksum, DeltaStore, DiffBlock, IndexedChecksumStore},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct InitialEmptyData;

#[derive(Serialize, Deserialize, Debug)]
pub struct ChunkProcessor<T> {
    chunk_size: usize,
    pub data: T,
}

impl ChunkProcessor<InitialEmptyData> {
    pub fn new(chunk_size: usize) -> Self {
        ChunkProcessor {
            chunk_size,
            data: InitialEmptyData,
        }
    }
}

impl<T> ChunkProcessor<T> {
    pub fn check_chunk_size_equal(&self, chunk_size: usize) -> Result<(), AppError> {
        if chunk_size == self.chunk_size {
            Ok(())
        } else {
            Err(AppError::IncompatibleChunkSize(
                format!("Current chunk size differes from the one for which signature was built: Please use {} value of the chunk_size parameter", self.chunk_size)
                    .to_string(),
            ))
        }
    }

    pub fn check_processing_data_size(&self, data_size: usize) -> Result<(), AppError> {
        if data_size >= self.chunk_size * 2 {
            Ok(())
        } else {
            Err(AppError::IncompatibleDataSize(format!("Too small chunk of data for processing: Please provide the file which has at list 2 * {} bytes", self.chunk_size).to_string()))
        }
    }
}

pub trait ChecksumProducer {
    fn produce_checksum(&self, data: Vec<u8>) -> Result<ChunkProcessor<ChecksumStore>, AppError>;
}

impl ChecksumProducer for ChunkProcessor<InitialEmptyData> {
    fn produce_checksum(&self, data: Vec<u8>) -> Result<ChunkProcessor<ChecksumStore>, AppError> {
        self.check_processing_data_size(data.len())?;

        let mut checksum_store = ChecksumStore::new();

        for chunk in ChunkIter::new(&data, self.chunk_size).by_chunk() {
            let ad32 = chunk.ad32();
            let hash = chunk.hash()?;

            checksum_store.push(ChunkChecksum { ad32, hash });
        }

        Ok(ChunkProcessor {
            chunk_size: self.chunk_size,
            data: checksum_store,
        })
    }
}

pub trait IndexedChecksumProducer {
    fn produce_indexed_checksum(&self) -> ChunkProcessor<IndexedChecksumStore>;
}

impl IndexedChecksumProducer for ChunkProcessor<ChecksumStore> {
    fn produce_indexed_checksum(&self) -> ChunkProcessor<IndexedChecksumStore> {
        let mut checksum_indexed_store = IndexedChecksumStore::new();

        for (i, chunk_checksum) in self.data.iter().enumerate() {
            checksum_indexed_store.insert(chunk_checksum.ad32, (chunk_checksum.hash, i));
        }

        ChunkProcessor {
            chunk_size: self.chunk_size,
            data: checksum_indexed_store,
        }
    }
}

pub trait DeltaProducer {
    fn produce_delta(&self, new_data: Vec<u8>) -> Result<ChunkProcessor<DeltaStore>, AppError>;
}

impl DeltaProducer for ChunkProcessor<IndexedChecksumStore> {
    fn produce_delta(&self, new_data: Vec<u8>) -> Result<ChunkProcessor<DeltaStore>, AppError> {
        self.check_processing_data_size(new_data.len())?;
        let mut diffs = DeltaStore::new();
        let mut modified_buf = vec![];

        let mut iter = ChunkIter::new(&new_data, self.chunk_size).by_byte();

        while let Some(chunk) = iter.next() {
            let ad32 = chunk.ad32();

            if self.data.contains_key(&ad32) {
                let current_hash = chunk.hash()?;

                let (hash, index) = Some(
                    self.data
                        .get_vec(&ad32)
                        .ok_or(AppError::IndexCorrupted)?
                        .iter()
                        .filter(|(hash, _)| *hash == current_hash)
                        .collect::<Vec<&([u8; 32], usize)>>(),
                )
                .filter(|vec| vec.len() == 1)
                .map(|vec| (&vec[0].0, vec[0].1))
                .ok_or(AppError::IndexCorrupted)?;

                if chunk.hash()? == *hash {
                    diffs.insert(
                        index.clone(),
                        DiffBlock {
                            start: index * self.chunk_size,
                            offset: index * self.chunk_size + self.chunk_size,
                            is_mising: false,
                            buf: modified_buf.to_owned(),
                        },
                    );
                    iter.skip_chunks(1);
                    modified_buf.clear();
                }
            } else {
                modified_buf.push(chunk[0]);
            }
        }

        Ok(ChunkProcessor {
            chunk_size: self.chunk_size,
            data: diffs,
        })
    }
}

pub trait DeltaExtender {
    fn extend_missed_blocks(self, chunk_checksum: &ChecksumStore) -> ChunkProcessor<DeltaStore>;
}

impl DeltaExtender for ChunkProcessor<DeltaStore> {
    fn extend_missed_blocks(self, chunk_checksum: &ChecksumStore) -> ChunkProcessor<DeltaStore> {
        let mut delta = self.data;

        for i in 0..chunk_checksum.len() {
            if !delta.contains_key(&i) {
                delta.insert(
                    i,
                    DiffBlock {
                        start: i * self.chunk_size,
                        offset: i * self.chunk_size + self.chunk_size,
                        is_mising: true,
                        buf: vec![],
                    },
                );
            }
        }

        ChunkProcessor {
            chunk_size: self.chunk_size,
            data: delta,
        }
    }
}

#[cfg(test)]
fn calculate_delta(data: Vec<u8>, new_data: Vec<u8>, chunk_size: usize) -> DeltaStore {
    let chunk_processor = ChunkProcessor::new(chunk_size);

    let checksum = chunk_processor.produce_checksum(data).unwrap();

    let indexed_checksum = checksum.produce_indexed_checksum();

    indexed_checksum
        .produce_delta(new_data)
        .unwrap()
        .extend_missed_blocks(&checksum.data)
        .data
}

#[cfg(test)]
fn check_delta_match(delta: DeltaStore, expected_delta: HashMap<usize, Vec<u8>>) {
    use std::str;
    for (index, lit) in &expected_delta {
        assert!(delta.contains_key(index));

        let res = delta.get(index).unwrap();
        // For better debug visibility converted to string
        assert_eq!(
            str::from_utf8(&res.buf).unwrap(),
            str::from_utf8(lit).unwrap()
        );
    }
}

#[test]
fn test_index_creation() {
    let message = "hello world I am testing index creation"
        .as_bytes()
        .to_vec();

    let chunk_processor = ChunkProcessor::new(4);

    let checksum = chunk_processor.produce_checksum(message).unwrap();
    let indexed_checksum = checksum.produce_indexed_checksum().data;

    for (i, chunk_checksum) in checksum.data.iter().enumerate() {
        assert!(indexed_checksum.contains_key(&chunk_checksum.ad32));

        let (hash, index) = Some(
            indexed_checksum
                .get_vec(&chunk_checksum.ad32)
                .unwrap()
                .iter()
                .filter(|(hash, _)| *hash == chunk_checksum.hash)
                .collect::<Vec<&([u8; 32], usize)>>(),
        )
        .filter(|vec| vec.len() == 1)
        .map(|vec| (&vec[0].0, vec[0].1))
        .unwrap();

        assert_eq!(index, i);
        assert_eq!(*hash, chunk_checksum.hash);
    }
}

#[test]
fn test_chunk_change() {
    let original =
        "i am here guys how are you doing this is a small test for chunk split and rolling hash"
            .as_bytes()
            .to_vec();

    let new_data = "i here guys how are you doing this is a mall test chunk split and rolling hash"
        .as_bytes()
        .to_vec();

    let expected_delta = HashMap::from([
        (1, "i here guys h".as_bytes().to_vec()), // Match first chunk change,
        (4, " this is a mall test chunk ".as_bytes().to_vec()), // Match chunk 4 changed
    ]);

    let delta = calculate_delta(original, new_data, 16);
    check_delta_match(delta, expected_delta);
}

#[test]
fn test_chunk_added() {
    let original =
        "i am here guys how are you doing this is a small test for chunk split and rolling hash"
            .as_bytes()
            .to_vec();

    let new_data = "i am here guys how are you doingadded this is a small test for chunk split and rolling hash"
        .as_bytes()
        .to_vec();

    let expected_delta = HashMap::from([
        (2, "added".as_bytes().to_vec()), // Match chunks 2 changed,
    ]);

    let delta = calculate_delta(original, new_data, 16);
    check_delta_match(delta, expected_delta);
}

#[test]
fn test_chunk_removed() {
    let original =
        "i am here guys how are you doing this is a small test for chunk split and rolling hash"
            .as_bytes()
            .to_vec();

    let new_data = "ow are you doing this is a small split and rolling hash"
        .as_bytes()
        .to_vec();

    let delta = calculate_delta(original, new_data, 16);

    let chunk_first = delta.get(&0);
    let chunk_third = delta.get(&3);

    assert!(chunk_first.is_some());
    assert!(chunk_third.is_some());

    let chunk_first = chunk_first.unwrap();
    let chunk_third = chunk_third.unwrap();

    assert_eq!(chunk_first.is_mising, true);
    assert_eq!(chunk_third.is_mising, true);

    assert_eq!(chunk_first.start, 0);
    assert_eq!(chunk_first.offset, 16);

    assert_eq!(chunk_third.start, 48);
    assert_eq!(chunk_third.offset, 64);
}

#[test]
fn test_chunk_shifted() {
    let original =
        "i am here guys how are you doing this is a small test for chunk split and rolling hash"
            .as_bytes()
            .to_vec();

    let new_data = "i am here guys   how are you doing    test for chunk split and rolling hash"
        .as_bytes()
        .to_vec();

    let expected_delta = HashMap::from([
        (1, "i am here guys   h".as_bytes().to_vec()), // Match 1 chunk changed,
        (3, "   ".as_bytes().to_vec()),                // Match 3 chunk change
    ]);

    let delta = calculate_delta(original, new_data, 16);
    check_delta_match(delta, expected_delta);
}
