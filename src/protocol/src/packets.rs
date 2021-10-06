use crate::varnum::VarInt;
use mckerel_protocol_macros::{enum_impl, Packet};

pub mod serverbound {
    use super::*;
    pub mod handshake {
        use super::*;
        pub mod packets {
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
            
        
            pub struct LegacyHandshake;
        }

        pub enum Packet {
            Handshake(packets::Handshake),
            LegacyHandshake(packets::LegacyHandshake),
        }
    }

    pub use handshake::Packet as HandshakePacket;

    pub mod status {
        use super::*;
        pub mod packets {
            use super::*;
            pub struct Request;

            pub struct Ping(u64);
        }

        pub enum Packet {
            Request(packets::Request),
            Ping(packets::Ping),
        }
    }

    pub use status::Packet as StatusPacket;
}

pub mod clientbound {
    pub mod status {
        use super::*;
        pub mod packets {
            use super::*;
            pub struct Response {
                pub resp: String
            }
    
            pub struct Pong(u64);
        }

        pub enum Packet {
            Response(packets::Response),
            Pong(packets::Pong),
        }
    }

    pub use status::Packet as StatusPacket;
}