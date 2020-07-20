use primitive_types::{H160, H256, U256};
use std::convert::TryInto;

const WORD_SIZE: usize = 32;

pub struct AbiArgIterator<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> AbiArgIterator<'a> {
    pub fn new<'b>(data: &'b [u8]) -> AbiArgIterator<'b> {
        AbiArgIterator { data, offset: 0 }
    }

    pub fn is_ended(&self) -> bool {
        self.offset == self.data.len()
    }

    pub fn next_byte32(&mut self) -> Option<&'a [u8]> {
        if self.offset < self.data.len() {
            let ret = &self.data[self.offset..self.offset + WORD_SIZE];
            self.offset += WORD_SIZE;
            Some(ret)
        } else {
            None
        }
    }

    pub fn next_words_as_bytes(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.offset < self.data.len() {
            let ret = &self.data[self.offset..self.offset + n * WORD_SIZE];
            self.offset += n * WORD_SIZE;
            Some(ret)
        } else {
            None
        }
    }

    pub fn next_u256(&mut self) -> Option<U256> {
        self.next_byte32().map(U256::from_big_endian)
    }

    pub fn next_h256(&mut self) -> Option<H256> {
        self.next_byte32().map(H256::from_slice)
    }

    pub fn next_h160(&mut self) -> Option<H160> {
        self.next_h256().map(From::from)
    }

    pub fn next_bytes(&mut self) -> Option<&'a [u8]> {
        let local_offset: usize = self.next_u256()?.try_into().ok()?;

        let size: usize = U256::from_big_endian(&self.data[local_offset..local_offset + WORD_SIZE])
            .try_into()
            .ok()?;
        Some(&self.data[local_offset + WORD_SIZE..local_offset + WORD_SIZE + size])
    }

    pub fn next_array_of_bytes(&mut self) -> Option<Vec<&'a [u8]>> {
        // memory offset
        let mut local_offset: usize = self.next_u256()?.try_into().ok()?;

        if local_offset < self.data.len() {
            let len: usize = U256::from_big_endian(&self.data[local_offset..local_offset + WORD_SIZE])
                .try_into()
                .ok()?;
            local_offset += WORD_SIZE;

            let mut inner = AbiArgIterator::new(&self.data[local_offset..]);
            (0..len).map(|_| inner.next_bytes()).collect()
        } else {
            Some(vec![])
        }
    }

    pub fn next_array_of_byte32(&mut self) -> Option<Vec<&'a [u8]>> {
        // memory offset
        let mut local_offset: usize = self.next_u256()?.try_into().ok()?;

        if local_offset < self.data.len() {
            let len: usize = U256::from_big_endian(&self.data[local_offset..local_offset + WORD_SIZE])
                .try_into()
                .ok()?;
            local_offset += WORD_SIZE;

            let mut inner = AbiArgIterator::new(&self.data[local_offset..]);
            (0..len).map(|_| inner.next_byte32()).collect()
        } else {
            Some(vec![])
        }
    }
}
