use num;
use tokio::io::{AsyncRead, AsyncReadExt};
use byteorder::ReadBytesExt;

pub struct ReaderState<T: num::PrimInt> {
    pub val: T,
    pub length: usize,
}

pub enum VarNumReader<T: num::PrimInt> {
    NotDone(ReaderState<T>),
    Done(Result<ReaderState<T>, ()>)
}

impl<T: num::PrimInt> ReaderState<T> {
    pub fn new() -> Self {
        Self {
            val: T::zero(),
            length: 0,
        }
    }
    
    pub fn read_byte(mut self, byte: u8) -> VarNumReader<T> {
        let done = byte & 0x80 == 0;
        let byte = T::from(byte & 0x7f).unwrap();

        let shift = 7 * self.length;
        if shift > byte.leading_zeros() as usize {
            return VarNumReader::Done(Err(()));
        }
        self.val = self.val | (byte << shift);
        self.length += 1;

        if done {
            VarNumReader::Done(Ok(self))
        }
        else {
            VarNumReader::NotDone(self)
        }
    }
}

impl<T: num::PrimInt> VarNumReader<T> {
    pub fn new() -> Self {
        Self::NotDone(ReaderState::new())
    }

    pub fn try_read_byte(self, byte: u8) -> Self {
        match self {
            Self::NotDone(state) => state.read_byte(byte),
            Self::Done(_) => panic!("tried to read into an already Done VarNumReader")
        }
    }

    pub fn read_from_get_state<R: std::io::Read>(mut self, mut read: R) -> std::io::Result<ReaderState<T>> {
        let res = loop {
            match self {
                Self::Done(res) => {
                    break res
                },
                Self::NotDone(state) => {
                    self = state.read_byte(read.read_u8()?);
                }
            }
        };
        res.map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to read varint"))
    }

    pub fn read_from<R: std::io::Read>(self, read: R) -> std::io::Result<T> {
        Ok(self.read_from_get_state(read)?.val)
    }

    pub async fn read_from_async_get_state<R: AsyncRead + std::marker::Unpin>(mut self, mut read: R) -> std::io::Result<ReaderState<T>> {
        let res = loop {
            match self {
                Self::Done(res) => break res,
                Self::NotDone(state) => {
                    self = state.read_byte(read.read_u8().await?);
                }
            }
        };
        res.map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "failed to read varint"))
    }

    pub async fn read_from_async<R: AsyncRead + std::marker::Unpin>(self, read: R) -> std::io::Result<T> {
        Ok(self.read_from_async_get_state(read).await?.val)
    }
}

pub type VarInt = VarNumReader<i32>;
pub type VarLong = VarNumReader<i64>;