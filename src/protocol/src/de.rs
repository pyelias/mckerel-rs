use num;
use crate::varnum::{VarNumReader, VarInt};

#[derive(Debug)]
pub enum Error {
    Eof,
    BadPacketId,
    BadEnumTag,
    BadVarNum,
    BadUtf8,
}

pub type Result<V> = std::result::Result<V, Error>;

pub struct ByteReader<'a> {
    input: &'a [u8]
}

impl<'a> ByteReader<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        match self.input.split_first() {
            Some((byte, input)) => {
                self.input = input;
                Ok(*byte)
            },
            None => Err(Error::Eof)
        }
    }

    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if len > self.remaining_len() {
            return Err(Error::Eof);
        }

        let (res, input) = self.input.split_at(len);
        self.input = input;
        Ok(res)
    }

    pub fn remaining_len(&self) -> usize {
        self.input.len()
    }

    pub fn done(&self) -> bool {
        self.remaining_len() == 0
    }
}

impl std::io::Read for ByteReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.done() {
            // returning a length of 0 means the reader is done
            // this would happen anyways, but it's best to be explicit
            // (probably maybe it gets optimized away)
            return Ok(0);
        }

        let len = std::cmp::min(buf.len(), self.remaining_len());
        buf.copy_from_slice(self.read_bytes(len).unwrap()); // can't error, prev. line ensures it's long enough
        Ok(len)
    }
}

pub trait Deserialize<'de> {
    type Value;

    fn deserialize(input: &mut ByteReader<'de>) -> Result<Self::Value>;
}

macro_rules! impl_deserialize_int {
    ($t: ty, $size: literal) => {
        impl Deserialize<'_> for $t {
            type Value = Self;

            fn deserialize(input: &mut ByteReader<'_>) -> Result<Self::Value> {
                use std::convert::TryInto;

                let bytes = input.read_bytes($size)?;
                let bytes_arr = bytes.try_into().unwrap(); // unwrap is ok, we know it's the right size
                Ok(<$t>::from_le_bytes(bytes_arr))
            }
        }
    };
}

impl_deserialize_int!(u8, 1);
impl_deserialize_int!(i8, 1);
impl_deserialize_int!(u16, 2);
impl_deserialize_int!(i16, 2);
impl_deserialize_int!(u32, 4);
impl_deserialize_int!(i32, 4);
impl_deserialize_int!(u64, 6);
impl_deserialize_int!(i64, 6);

impl<T: num::PrimInt> Deserialize<'_> for VarNumReader<T> {
    type Value = T;

    fn deserialize(input: &mut ByteReader<'_>) -> Result<Self::Value> {
        let mut reader = Self::new();
        let res = loop {
            match reader {
                Self::Done(res) => break res,
                Self::NotDone(state) => {
                    let byte = input.read_byte()?;
                    reader = state.read_byte(byte);
                }
            }
        };
        res.map(|r| r.val).map_err(|_| Error::BadVarNum)
    }
}

impl<'de> Deserialize<'de> for &'de str {
    type Value = &'de str;

    fn deserialize(input: &mut ByteReader<'de>) -> Result<Self::Value> {
        let len = VarInt::deserialize(input)? as usize;
        let bytes = input.read_bytes(len)?;
        Ok(std::str::from_utf8(bytes).map_err(|_| Error::BadUtf8)?)
    }
}

impl Deserialize<'_> for String {
    type Value = Self;

    fn deserialize<'de>(input: &mut ByteReader<'de>) -> Result<Self::Value> {
        Ok(<&'de str as Deserialize>::deserialize(input)?.to_owned())
    }
}