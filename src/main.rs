use mckerel_protocol;
use mckerel_protocol::de::Deserialize;
use tokio::net::{TcpListener, TcpStream};

async fn handle_connection(conn: TcpStream) {
    println!("got a connection");

    let (send, mut recv) = mckerel_protocol::make_conn(conn);

    let packet = match recv.read_packet_or_legacy_ping().await.unwrap() {
        mckerel_protocol::PacketOrLegacyPing::Packet(p) => p.read_all().await.unwrap(),
        mckerel_protocol::PacketOrLegacyPing::LegacyPing => {
            println!("got a legacy ping");
            vec![0xfe]
        }
    };
    println!("{:?}", packet);

    let mut content_deser = mckerel_protocol::de::ByteReader::new(&packet);
    let packet_data = mckerel_protocol::packets::serverbound::handshake::Packet::deserialize(&mut content_deser).unwrap( );
    if let mckerel_protocol::packets::serverbound::handshake::Packet::Handshake(packet_data) = packet_data {
        println!("{} {}", packet_data.version, packet_data.address);
    }
    else {
        println!("not the right kind of packet i guess");
    }

    send.shutdown();
}

#[tokio::main]
pub async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.2:25565").await?;

    loop {
        if let Ok((conn, _)) = listener.accept().await {
            tokio::spawn(async move { handle_connection(conn).await });
        }
    }
}
