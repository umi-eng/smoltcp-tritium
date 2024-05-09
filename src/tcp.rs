//! TCP protocol.

use crate::{
    dgram::{Frame, Header, Packet},
    heartbeat, BusNumber, HEARTBEAT_DURATION, PORT, PROTO_VER,
};
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    socket::tcp::{SendError, Socket, SocketBuffer, State},
    time::Instant,
    wire::EthernetAddress,
};

pub struct Server {
    // configuration
    handle: SocketHandle,
    mac_addr: [u8; 6],
    bus_number: BusNumber,
    data_rate: u16,

    // state
    last_heartbeat: Instant,
}

impl Server {
    pub fn new<'a>(
        sockets: &mut SocketSet<'a>,
        rx_buffer: SocketBuffer<'a>,
        tx_buffer: SocketBuffer<'a>,
        mac_addr: EthernetAddress,
        now: Instant,
        bus_number: BusNumber,
        data_rate: u16,
    ) -> Self {
        let socket = Socket::new(rx_buffer, tx_buffer);
        let handle = sockets.add(socket);

        Self {
            handle,
            mac_addr: mac_addr.0,
            last_heartbeat: now,
            bus_number,
            data_rate,
        }
    }

    pub fn poll(&mut self, sockets: &mut SocketSet, now: Instant) {
        let socket = sockets.get_mut::<Socket>(self.handle);

        if !socket.is_open() {
            if !socket.is_listening() {
                if let Err(_err) = socket.listen(PORT) {
                    #[cfg(feature = "defmt-03")]
                    defmt::error!("Failed to bind to {}: {}", PORT, _err);
                }
            }
        }

        // if client closes, close on our end as well
        if socket.state() == State::CloseWait {
            socket.close();
            return;
        }

        if socket.can_send() {
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
    }

    /// Send heartbeat.
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

        socket.send_slice(packet.as_bytes()).map(|_| ())
    }

    /// Send can frame.
    pub fn send_frame(
        &mut self,
        sockets: &mut SocketSet,
        frame: &impl embedded_can::Frame,
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

        socket.send_slice(packet.as_bytes()).map(|_| ())
    }
}
