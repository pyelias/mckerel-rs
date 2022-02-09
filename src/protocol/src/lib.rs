// has to be at the top because macro scoping is broken
#[macro_use]
mod macros;

pub mod connection;
pub mod packets;
pub mod de;
pub mod ser;
//pub mod states;
mod varnum;

pub use connection::{Recv, Send, PacketReader, PacketOrLegacyPing, make_conn};
pub use packets::{serverbound, clientbound};