use mckerel_protocol;
use mckerel_protocol::de::Deserialize;
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(conn: TcpStream) {
    println!("got a connection");

    let mut conn = mckerel_protocol::Connection::new(conn);
    conn.expect_legacy_ping = true;

    let packet = conn.read_packet().await.unwrap();
    println!("{:?}", packet.content);

    let mut content_deser = mckerel_protocol::de::ByteReader::new(&packet.content);
    let packet_data = mckerel_protocol::packets::serverbound::handshake::Packet::deserialize(&mut content_deser).unwrap();
    if let mckerel_protocol::packets::serverbound::handshake::Packet::Handshake(packet_data) = packet_data {
        println!("{} {}", packet_data.version, packet_data.address);
    }
    else {
        println!("not the right kind of packet i guess");
    }

    conn.close().await.unwrap();
}

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565").await?;

    loop {
        if let Ok((conn, _)) = listener.accept().await {
            tokio::spawn(async move { handle_connection(conn).await });
        }
    }
}
