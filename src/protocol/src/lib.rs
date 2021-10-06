pub mod connection;
pub mod packets;
pub mod de;
pub mod ser;
mod varnum;

pub use connection::Connection;
pub use packets::{serverbound, clientbound};