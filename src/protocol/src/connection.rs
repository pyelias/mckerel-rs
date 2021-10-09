use tokio::net::TcpStream;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufReader};
use crate::varnum::VarInt;

pub struct Packet {
    // pub id: u8,
    pub content: Box<[u8]>,
}

pub struct Connection {
    read: BufReader<OwnedReadHalf>,
    write: OwnedWriteHalf,
    pub expect_legacy_ping: bool,
    pub compression_threshold: Option<u32>,
}

impl Connection {
    pub fn new(conn: TcpStream) -> Self {
        let (read, write) = conn.into_split();
        let read = BufReader::with_capacity(1024, read);
        Self {
            read,
            write,
            expect_legacy_ping: false,
            compression_threshold: None,
        }
    }

    async fn read_legacy_ping(&mut self) -> io::Result<Packet> {
        let mut content = vec![0xfe];
        self.read.read_to_end(&mut content).await?;
        let content = content.into_boxed_slice();
        Ok(Packet { content })
    }

    pub async fn read_packet(&mut self) -> io::Result<Packet> {
        let mut packet_length_reader = VarInt::new();
        if self.expect_legacy_ping {
            let first_byte = self.read.read_u8().await?;
            if first_byte == 0xfe {
                return self.read_legacy_ping().await;
            }
            packet_length_reader = packet_length_reader.try_read_byte(first_byte);
        }
        let packet_length = packet_length_reader.read_from_async(&mut self.read).await? as usize;

        let mut content = vec![0; packet_length];
        self.read.read_exact(&mut content).await?;
        let content = content.into_boxed_slice();

        Ok(Packet { content })
    }

    pub async fn close(&mut self) -> io::Result<()> {
        self.write.shutdown().await?;
        Ok(())
    }
}