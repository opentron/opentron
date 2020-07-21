use primitive_types::{H160, H256, U256};
use std::convert::TryInto;

use super::Error;

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

    pub fn next_byte32(&mut self) -> Result<&'a [u8], Error> {
        if self.offset < self.data.len() {
            let ret = &self.data[self.offset..self.offset + WORD_SIZE];
            self.offset += WORD_SIZE;
            Ok(ret)
        } else {
            Err(Error::AbiDecode)
        }
    }

    pub fn next_byte32_as_array(&mut self) -> Result<[u8; 32], Error> {
        let mut ret = [0u8; WORD_SIZE];
        ret.copy_from_slice(self.next_byte32()?);
        Ok(ret)
    }

    pub fn next_fixed_words(&mut self, n: usize) -> Result<&'a [u8], Error> {
        if self.offset < self.data.len() {
            let ret = &self.data[self.offset..self.offset + n * WORD_SIZE];
            self.offset += n * WORD_SIZE;
            Ok(ret)
        } else {
            Err(Error::AbiDecode)
        }
    }

    pub fn next_u256(&mut self) -> Result<U256, Error> {
        self.next_byte32().map(U256::from_big_endian)
    }

    pub fn next_h256(&mut self) -> Result<H256, Error> {
        self.next_byte32().map(H256::from_slice)
    }

    pub fn next_h160(&mut self) -> Result<H160, Error> {
        self.next_h256().map(From::from)
    }

    pub fn next_bytes(&mut self) -> Result<&'a [u8], Error> {
        let local_offset: usize = self.next_u256()?.try_into().map_err(|_| Error::AbiDecode)?;

        let size: usize = U256::from_big_endian(&self.data[local_offset..local_offset + WORD_SIZE])
            .try_into()
            .map_err(|_| Error::AbiDecode)?;
        Ok(&self.data[local_offset + WORD_SIZE..local_offset + WORD_SIZE + size])
    }

    pub fn next_array_of_bytes(&mut self) -> Result<Vec<&'a [u8]>, Error> {
        // memory offset
        let mut local_offset: usize = self.next_u256()?.try_into().map_err(|_| Error::AbiDecode)?;

        if local_offset < self.data.len() {
            let len: usize = U256::from_big_endian(&self.data[local_offset..local_offset + WORD_SIZE])
                .try_into()
                .map_err(|_| Error::AbiDecode)?;
            local_offset += WORD_SIZE;

            let mut inner = AbiArgIterator::new(&self.data[local_offset..]);
            (0..len).map(|_| inner.next_bytes()).collect()
        } else {
            Ok(vec![])
        }
    }

    pub fn next_array_of_byte32(&mut self) -> Result<Vec<&'a [u8]>, Error> {
        self.next_array_of_fixed_words(1)
    }

    pub fn next_array_of_fixed_words(&mut self, n: usize) -> Result<Vec<&'a [u8]>, Error> {
        // memory offset
        let mut local_offset: usize = self.next_u256()?.try_into().map_err(|_| Error::AbiDecode)?;

        if local_offset < self.data.len() {
            let len: usize = U256::from_big_endian(&self.data[local_offset..local_offset + WORD_SIZE])
                .try_into()
                .map_err(|_| Error::AbiDecode)?;
            local_offset += WORD_SIZE;

            let mut inner = AbiArgIterator::new(&self.data[local_offset..]);
            (0..len).map(|_| inner.next_fixed_words(n)).collect()
        } else {
            Ok(vec![])
        }
    }
}
