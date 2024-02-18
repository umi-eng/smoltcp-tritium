//! UDP server

use crate::{
    BusNumber, Flags, BCAST_ADDR, HEARTBEAT_DURATION, PORT, PROTO_VER,
};
use embedded_can::{Frame, Id};
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    phy::PacketMeta,
    socket::udp::{PacketBuffer, SendError, Socket, UdpMetadata},
    time::Instant,
    wire::{EthernetAddress, IpEndpoint},
};

bitfield::bitfield! {
    struct Packet(MSB0 [u8]);
    impl Debug;
    pub u64, version, set_version: 59, 8;
    pub u8, bus_number, set_bus_number: 63, 60;
    pub u64, client_identifier, set_client_identifier: 127, 72;
    pub u32, can_identifier, set_can_identifier: 159, 128;
    pub u8, flags, set_flags: 167, 160;
    pub u8, can_length, set_can_length: 175, 168;
    pub u64, can_data, set_can_data: 239, 176;
}

const PACKET_LEN: usize = 30;

/// Server instance.
pub struct Server {
    // configuration
    handle: SocketHandle,
    meta: UdpMetadata,
    mac_addr: [u8; 6],
    bus_number: BusNumber,
    data_rate: u16,

    // state
    last_heartbeat: Instant,
}

/// Error kind.
pub enum Error {}

impl Server {
    /// Creates a new [`Server`] instance.
    pub fn new<'a>(
        sockets: &mut SocketSet<'a>,
        rx_buffer: PacketBuffer<'a>,
        tx_buffer: PacketBuffer<'a>,
        mac_addr: EthernetAddress,
        now: Instant,
        bus_number: BusNumber,
        data_rate: u16,
    ) -> Result<Server, Error> {
        let socket = Socket::new(rx_buffer, tx_buffer);
        let handle = sockets.add(socket);

        let meta = UdpMetadata {
            endpoint: IpEndpoint {
                addr: BCAST_ADDR,
                port: PORT,
            },
            meta: PacketMeta::default(),
        };

        Ok(Server {
            handle,
            meta,
            mac_addr: mac_addr.0,
            bus_number,
            data_rate,
            last_heartbeat: now,
        })
    }

    /// Get the current bus number.
    pub fn bus_number(&self) -> BusNumber {
        self.bus_number
    }

    /// Set a new bus number.
    pub fn set_bus_number(&mut self, bus_number: BusNumber) {
        self.bus_number = bus_number;
    }

    /// Perform bufferred transactions and send heartbeat if needed.
    ///
    /// This function should be called at least every 10ms to keep up with traffic.
    pub fn poll(&mut self, sockets: &mut SocketSet, now: Instant) {
        let socket = sockets.get_mut::<Socket>(self.handle);

        if !socket.is_open() {
            match socket.bind(PORT) {
                Ok(_) => {}
                Err(_err) => {
                    #[cfg(feature = "defmt-03")]
                    defmt::error!("Failed binding to port {}: {}", PORT, _err);
                }
            }
        }

        if now - self.last_heartbeat > HEARTBEAT_DURATION {
            match self.write_heartbeat(socket) {
                Ok(_) => self.last_heartbeat = now,
                Err(_err) => {
                    #[cfg(feature = "defmt-03")]
                    defmt::error!("Failed to send heartbeat: {}", _err);
                }
            }
        }
    }

    /// Broadcast heartbeat.
    ///
    /// Note: this doesn't reset the heartbeat interval.
    pub fn send_heartbeat(
        &mut self,
        sockets: &mut SocketSet,
    ) -> Result<(), SendError> {
        let socket = sockets.get_mut::<Socket>(self.handle);

        self.write_heartbeat(socket)
    }

    fn write_heartbeat(&self, socket: &mut Socket) -> Result<(), SendError> {
        let flags = Flags::Heartbeat;

        let mut data = [0u8; 8];
        // bitrate
        data[0..2].copy_from_slice(&self.data_rate.to_be_bytes());
        data[2..8].copy_from_slice(&self.mac_addr);

        let mut packet = Packet([0; PACKET_LEN]);
        // metadata
        packet.set_version(PROTO_VER);
        packet.set_bus_number(self.bus_number.0);
        packet.set_client_identifier(u64::from_be_bytes([0u8; 8]));
        packet.set_flags(flags.bits());
        // frame
        packet.set_can_identifier(0);
        packet.set_can_length(8);
        packet.set_can_data(u64::from_be_bytes(data));

        socket.send_slice(&packet.0, self.meta)
    }

    /// Broadcast a CAN frame.
    pub fn send_frame(
        &mut self,
        sockets: &mut SocketSet,
        frame: &impl Frame,
    ) -> Result<(), SendError> {
        let socket = sockets.get_mut::<Socket>(self.handle);

        self.write_frame(socket, frame)
    }

    pub fn write_frame(
        &self,
        socket: &mut Socket,
        frame: &impl Frame,
    ) -> Result<(), SendError> {
        if frame.dlc() > 8 {
            // we only support standard frames of up to 8 bytes in length.
            return Ok(()); // todo: error
        }

        let mut flags = Flags::empty();

        if frame.is_extended() {
            flags.insert(Flags::Extended);
        }

        if frame.is_remote_frame() {
            flags.insert(Flags::Remote);
        }

        let id = match frame.id() {
            Id::Standard(id) => id.as_raw() as u32,
            Id::Extended(id) => id.as_raw(),
        };

        let mut packet = Packet([0; PACKET_LEN]);
        // metadata
        packet.set_version(PROTO_VER);
        packet.set_bus_number(self.bus_number.0);
        packet.set_client_identifier(u64::from_be_bytes([0u8; 8]));
        packet.set_flags(flags.bits());
        // can frame
        packet.set_can_identifier(id);
        packet.set_can_length(frame.dlc() as u8);
        packet.set_can_data(0);

        socket.send_slice(&packet.0, self.meta)
    }
}
