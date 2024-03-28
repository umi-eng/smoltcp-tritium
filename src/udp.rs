//! UDP protocol.

use crate::{
    dgram::{Frame, Header, Packet},
    heartbeat, BusNumber, BCAST_ADDR, HEARTBEAT_DURATION, PORT, PROTO_VER,
};
use embedded_can::Frame as CanFrame;
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    phy::PacketMeta,
    socket::udp::{PacketBuffer, SendError, Socket, UdpMetadata},
    time::Instant,
    wire::{EthernetAddress, IpEndpoint},
};

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
    ) -> Server {
        let socket = Socket::new(rx_buffer, tx_buffer);
        let handle = sockets.add(socket);

        let meta = UdpMetadata {
            endpoint: IpEndpoint {
                addr: BCAST_ADDR,
                port: PORT,
            },
            meta: PacketMeta::default(),
        };

        Server {
            handle,
            meta,
            mac_addr: mac_addr.0,
            bus_number,
            data_rate,
            last_heartbeat: now,
        }
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
        let packet =
            heartbeat::build(&self.mac_addr, &self.bus_number, &self.data_rate);

        socket.send_slice(packet.as_bytes(), self.meta)
    }

    /// Broadcast a CAN frame.
    pub fn send_frame(
        &mut self,
        sockets: &mut SocketSet,
        frame: &impl CanFrame,
    ) -> Result<(), SendError> {
        let socket = sockets.get_mut::<Socket>(self.handle);

        let mut packet = Packet {
            header: Header::new(),
            frame: Frame::from_frame(frame).unwrap(),
        };
        packet.header.set_version(PROTO_VER);
        packet.header.set_bus_number(self.bus_number.0);
        packet
            .header
            .set_client_identifier(u64::from_be_bytes([0u8; 8]));

        socket.send_slice(packet.as_bytes(), self.meta)
    }
}
