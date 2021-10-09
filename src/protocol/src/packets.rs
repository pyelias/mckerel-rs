use crate::varnum::VarInt;
use crate::macros::{enum_impl, Packet}; // don't use packets_impl because macro scoping is broken

pub mod serverbound {
    use super::*;
    pub mod handshake {
        use super::*;

        enum_impl!(VarInt HandshakeNextState {
            Status = 1,
            Login = 2
        });


        #[derive(Packet)]
        pub struct Handshake {
            #[packet(with = "VarInt")]
            pub version: i32,
            pub address: String,
            pub port: u16,
            pub next_state: HandshakeNextState
        }
    
        #[derive(Packet)]
        pub struct LegacyPing;

        pub enum Packet {
            Handshake(Handshake),
            LegacyPing(LegacyPing)
        }

        // have to manually implement deserialize on handshake packets
        // since LegacyPing has a legacy format
        impl<'de> crate::de::Deserialize<'de> for Packet {
            type Value = Self;

            fn deserialize(mut input: &mut crate::de::ByteReader<'de>) -> crate::de::Result<Self::Value> {
                let id_reader = VarInt::new();
                let first_byte = input.read_byte()?;
                if first_byte == 0xfe {
                    return Ok(Self::LegacyPing(LegacyPing))
                }
                let id = id_reader.try_read_byte(first_byte).read_from(&mut input);
                let id = id.map_err(|_| crate::de::Error::Eof)?; // ByteReaders can only return eof errors
                match id {
                    0 => Ok(Self::Handshake(Handshake::deserialize(input)?)),
                    _ => Err(crate::de::Error::BadPacketId),
                }
            }
        }
    }

    pub mod status {
        use super::*;

        #[derive(Packet)]
        pub struct Request;
        
        #[derive(Packet)]
        pub struct Ping(u64);

        packets_impl!(Packet {
            Request = 0x00,
            Ping = 0x01
        });
    }
}

pub mod clientbound {
    use super::*;
    pub mod status {
        use super::*;

        #[derive(Packet)]
        pub struct Response {
            pub resp: String
        }

        #[derive(Packet)]
        pub struct Pong(u64);

        packets_impl!(Packet {
            Response = 0x00,
            Pong = 0x01
        });
    }
}