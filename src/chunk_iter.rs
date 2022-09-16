#[derive(Default)]
pub struct DefaultIter {}

#[derive(Default)]
pub struct ByChunkIter {}

#[derive(Default)]
pub struct ByByteIter {}

impl ByByteIter {
    const STEP: usize = 1;

    fn step(&self) -> usize {
        ByByteIter::STEP
    }
}

pub struct ChunkIter<'a, T> {
    value: &'a Vec<u8>,
    chunk_size: usize,
    index: usize,
    type_iter: T,
}

impl<'a> ChunkIter<'a, DefaultIter> {
    pub fn new(value: &'a Vec<u8>, chunk_size: usize) -> ChunkIter<'a, DefaultIter> {
        ChunkIter::<'a, DefaultIter> {
            value,
            chunk_size,
            index: 0,
            type_iter: DefaultIter::default(),
        }
    }

    pub fn by_chunk(self) -> ChunkIter<'a, ByChunkIter> {
        ChunkIter::<'a, ByChunkIter> {
            value: self.value,
            chunk_size: self.chunk_size,
            index: self.index,
            type_iter: ByChunkIter::default(),
        }
    }

    pub fn by_byte(self) -> ChunkIter<'a, ByByteIter> {
        ChunkIter::<'a, ByByteIter> {
            value: self.value,
            chunk_size: self.chunk_size,
            index: self.index,
            type_iter: ByByteIter::default(),
        }
    }
}

impl<'a> Iterator for ChunkIter<'a, ByChunkIter> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let start_index = self.index * self.chunk_size;
        self.index += 1;

        if start_index < self.value.len() {
            let end_index = if (start_index + self.chunk_size) <= self.value.len() {
                start_index + self.chunk_size
            } else {
                start_index + (self.value.len() - start_index)
            };

            Some(&self.value[start_index..end_index])
        } else {
            None
        }
    }
}

impl<'a> Iterator for ChunkIter<'a, ByByteIter> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        let start_index = self.index * self.type_iter.step();
        let end_index = start_index + self.chunk_size;
        self.index += 1;

        if end_index <= self.value.len() {
            Some(&self.value[start_index..end_index])
        } else {
            None
        }
    }
}

pub trait SkipChunk {
    fn skip_chunks(&mut self, n: usize);
}

impl<'a> SkipChunk for ChunkIter<'a, ByByteIter> {
    fn skip_chunks(&mut self, n: usize) {
        self.index += n * self.chunk_size - 1;
    }
}

#[test]
fn check_basic_iteration() {
    let value: Vec<u8> = vec![1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4];
    let chunk_size = 4;

    let mut counter = 0;
    for (i, el) in ChunkIter::new(&value, chunk_size).by_chunk().enumerate() {
        assert_eq!(el.to_vec(), vec![1, 2, 3, 4]);
        assert_eq!(counter, i);

        counter += 1;
    }

    assert_eq!(counter, 3);
}

#[test]
fn check_last_chunk_shorter_iteration() {
    let value: Vec<u8> = vec![1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3];
    let chunk_size = 4;

    let mut counter = 0;
    for (i, el) in ChunkIter::new(&value, chunk_size).by_chunk().enumerate() {
        if i == 3 {
            assert_eq!(el.to_vec(), vec![1, 2, 3]);
        } else {
            assert_eq!(el.to_vec(), vec![1, 2, 3, 4]);
        }
        assert_eq!(counter, i);
        counter += 1;
    }

    assert_eq!(counter, 4);
}

#[test]
fn check_by_byte_chunk_iteration() {
    let value: Vec<u8> = vec![1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3];
    let chunk_size = 4;

    let mut counter = 0;
    for (i, el) in ChunkIter::new(&value, chunk_size).by_byte().enumerate() {
        let mut expected = vec![1, 2, 3, 4];
        let len = expected.len();
        expected.rotate_left(i % len);

        assert_eq!(el.to_vec(), expected);
        assert_eq!(counter, i);

        counter += 1;
    }

    assert_eq!(counter, 12);
}

#[test]
fn check_by_byte_chunk_iteration_with_skip() {
    let value: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6];
    let chunk_size = 4;

    let mut iter = ChunkIter::new(&value, chunk_size).by_byte();
    assert_eq!(iter.next().unwrap().to_vec(), vec![1, 2, 3, 4]);
    assert_eq!(iter.next().unwrap().to_vec(), vec![2, 3, 4, 5]);
    assert_eq!(iter.next().unwrap().to_vec(), vec![3, 4, 5, 6]);
    assert_eq!(iter.next().unwrap().to_vec(), vec![4, 5, 6, 7]);
    assert_eq!(iter.next().unwrap().to_vec(), vec![5, 6, 7, 8]);
    iter.skip_chunks(1);
    assert_eq!(iter.next().unwrap().to_vec(), vec![9, 1, 2, 3]);
    assert_eq!(iter.next().unwrap().to_vec(), vec![1, 2, 3, 4]);
    assert_eq!(iter.next().unwrap().to_vec(), vec![2, 3, 4, 5]);
    assert_eq!(iter.next().unwrap().to_vec(), vec![3, 4, 5, 6]);
    iter.skip_chunks(1);
    assert_eq!(iter.next(), None);
}
