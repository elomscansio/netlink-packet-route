use netlink_packet_route::{
    NeighbourMessage, NetlinkFlags, NetlinkHeader, NetlinkMessage, NetlinkPayload, RtnlMessage,
    NLM_F_DUMP, NLM_F_REQUEST,
};
use netlink_sys::{Protocol, Socket, SocketAddr};

fn main() {
    let mut socket = Socket::new(Protocol::Route).unwrap();
    let _port_number = socket.bind_auto().unwrap().port_number();
    socket.connect(&SocketAddr::new(0, 0)).unwrap();

    let header = NetlinkHeader::new();
    let rtnl_message = RtnlMessage::GetNeighbour(NeighbourMessage::default());
    let mut packet = NetlinkMessage::new(header, NetlinkPayload::from(rtnl_message));

    packet.header.flags = NetlinkFlags::from(NLM_F_DUMP | NLM_F_REQUEST);
    packet.header.sequence_number = 1;
    packet.finalize();
    let mut buf = vec![0; packet.header.length as usize];

    assert!(buf.len() == packet.buffer_len());
    packet.serialize(&mut buf[..]);

    println!(">>> {:?}", packet);
    socket.send(&buf[..], 0).unwrap();

    let mut receive_buffer = vec![0; 4096];
    let mut offset = 0;

    loop {
        let size = socket.recv(&mut receive_buffer[..], 0).unwrap();

        loop {
            let bytes = &receive_buffer[offset..];
            let rx_packet: NetlinkMessage<RtnlMessage> =
                NetlinkMessage::deserialize(bytes).unwrap();

            println!("<<< {:?}", rx_packet);

            if rx_packet.payload == NetlinkPayload::Done {
                println!("Done!");
                return;
            }

            offset += rx_packet.header.length as usize;
            if offset == size || rx_packet.header.length == 0 {
                offset = 0;
                break;
            }
        }
    }
}