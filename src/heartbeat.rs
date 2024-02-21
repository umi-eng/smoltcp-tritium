use crate::dgram::{Frame, Header, Packet};
use crate::{BusNumber, Flags, PROTO_VER};

/// Builds a heartbeat packet.
pub fn build(
    mac_addr: &[u8; 6],
    bus_number: &BusNumber,
    data_rate: &u16,
) -> Packet {
    let flags = Flags::Heartbeat;

    let mut data = [0u8; 8];
    // bitrate
    data[0..2].copy_from_slice(&data_rate.to_be_bytes());
    data[2..8].copy_from_slice(mac_addr);

    let mut packet = Packet {
        header: Header::new(),
        frame: Frame::new(),
    };

    // metadata
    packet.header.set_version(PROTO_VER);
    packet.header.set_bus_number(bus_number.0);
    packet
        .header
        .set_client_identifier(u64::from_be_bytes([0u8; 8]));

    // frame
    packet.frame.set_flags(flags.bits());
    packet.frame.set_id(0);
    packet.frame.set_dlc(data.len() as u8);
    packet.frame.set_data(u64::from_be_bytes(data));

    packet
}
