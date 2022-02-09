use std::task::{Poll, Context};
use std::pin::Pin;
use std::future::Future;
use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::{self, AsyncRead, AsyncReadExt, AsyncWriteExt, AsyncBufRead, AsyncBufReadExt, BufReader, ReadBuf};
use flate2;
use crate::varnum::VarInt;

struct ConnReaderInner {
    // would make a type alias for this, but cant think of a good name
    // ReadReader?
    read: OwnedReadHalf,
    // encryption too, later
}

impl AsyncRead for ConnReaderInner {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let read = Pin::new(&mut self.read);
        read.poll_read(cx, buf)
    }
}

type ConnReader = BufReader<ConnReaderInner>;

async fn async_decompress<R: AsyncBufRead + Unpin>(read: &mut R, decompress: &mut flate2::Decompress, dst: &mut [u8]) -> io::Result<usize> {
    // like flate2::zio::read but async
    if dst.is_empty() {
        return Ok(0);
    }
    loop {
        let compressed = read.fill_buf().await?;
        if compressed.is_empty() {
            return Ok(0);
        }
        let flush = if compressed.is_empty() {
            flate2::FlushDecompress::Finish
        } else {
            flate2::FlushDecompress::None
        };

        let old_total_in = decompress.total_in();
        let old_total_out = decompress.total_out();
        let decompress_result = decompress.decompress(compressed, dst, flush);
        let consumed = (decompress.total_in() - old_total_in) as usize;
        let produced = (decompress.total_out() - old_total_out) as usize;
        read.consume(consumed);

        let is_stream_end = match &decompress_result {
            Ok(flate2::Status::StreamEnd) => true,
            _ => false
        };
        if produced == 0 && !is_stream_end {
            continue;
        }

        return match decompress_result {
            Ok(_) => Ok(produced),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "corrupt deflate stream",
            ))
        };
    }

}

pub struct PacketReader<'a> {
    read: io::Take<&'a mut ConnReader>,
    decompress: Option<&'a mut flate2::Decompress>,
    length: usize
}

impl<'a> PacketReader<'a> {
    pub async fn read_all(mut self) -> io::Result<Vec<u8>> {
        let mut res = vec![0; self.length];
        self.read_exact(&mut res).await?;
        Ok(res)
    }
}

impl AsyncRead for PacketReader<'_> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let (decompress, read) = unsafe {
            let s = self.get_unchecked_mut();
            (&mut s.decompress, &mut s.read)
        };
        match decompress {
            Some(decompress) => {
                let read_into = buf.initialize_unfilled();

                let mut decompressing_future = async_decompress(read, decompress, read_into);
                let decompressing_future_pin = unsafe { Pin::new_unchecked(&mut decompressing_future) };
                let res = match decompressing_future_pin.poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(res) => res
                };
                // explicitly drop so buf becomes available again
                // right now, it's ok to just drop this future and cancel what it was doing
                // if it changes later, it might not be anymore
                std::mem::drop(decompressing_future);

                return Poll::Ready(match res {
                    Ok(len_read) => {
                        buf.advance(len_read);
                        Ok(())
                    },
                    Err(err) => Err(err)
                })
            },
            None => {
                return Pin::new(read).poll_read(cx, buf);
            }
        }
    }
}

pub enum PacketOrLegacyPing<'a> {
    Packet(PacketReader<'a>),
    // this will later have a reader in it or something
    LegacyPing
}

struct RecvCompression {
    decompress: flate2::Decompress
}

pub struct Recv {
    read: ConnReader,
    compression: Option<RecvCompression>,
}

impl Recv {
    pub fn new(read: OwnedReadHalf) -> Self {
        Self {
            read: BufReader::new(ConnReaderInner { read } ),
            compression: None,
        }
    }

    pub async fn read_packet_with_length(&mut self, mut packet_length: usize) -> io::Result<PacketReader<'_>> {
        let mut data_length = packet_length;
        let decompress = match &mut self.compression {
            None => None,
            Some(compression) => {
                // if compression is enabled, read the data length
                // and use compression if it's non-zero
                let data_length_info = VarInt::new().read_from_async_get_state(&mut self.read).await?;
                packet_length -= data_length_info.length as usize;
                data_length = data_length_info.val as usize;
    
                if data_length != 0 {
                    let decompress = &mut compression.decompress;
                    decompress.reset(true); // true means expect a zlib header, which will appear
                    Some(decompress)
                } else {
                    None
                }
            }
        };

        let read = (&mut self.read).take(packet_length as u64);
        Ok(PacketReader {
            read,
            decompress,
            length: data_length
        })
    }

    pub async fn read_packet(&mut self) -> io::Result<PacketReader<'_>> {
        let packet_length = VarInt::new().read_from_async(&mut self.read).await? as usize;
        self.read_packet_with_length(packet_length).await
    }

    // old clients may send an initial packet following a different format, so be able to handle those also
    pub async fn read_packet_or_legacy_ping(&mut self) -> io::Result<PacketOrLegacyPing<'_>> {
        let mut packet_length_reader = VarInt::new();
        let first_byte = self.read.read_u8().await?;
        if first_byte == 0xfe {
            return Ok(PacketOrLegacyPing::LegacyPing);
        }
        packet_length_reader = packet_length_reader.try_read_byte(first_byte);
        let packet_length = packet_length_reader.read_from_async(&mut self.read).await? as usize;
        Ok(PacketOrLegacyPing::Packet(self.read_packet_with_length(packet_length).await?))
    }
}

pub struct Send {
    write: OwnedWriteHalf
}

impl Send {
    pub fn new(write: OwnedWriteHalf) -> Self {
        Self { write }
    }

    pub fn shutdown(self) {
        // drop self.write
    }
}

// also will return a Send, eventually
pub fn make_conn(conn: TcpStream) -> (Send, Recv) {
    let (read, write) = conn.into_split();
    (Send::new(write), Recv::new(read))
}