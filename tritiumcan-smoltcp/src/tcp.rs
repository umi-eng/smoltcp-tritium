//! TCP protocol.

use core::mem::size_of;

use smoltcp::{
    iface::{SocketHandle, SocketSet},
    socket::tcp::{RecvError, SendError, Socket, SocketBuffer, State},
    time::Instant,
    wire::EthernetAddress,
};
use tritiumcan::{
    datagram::{Frame, Packet},
    BusNumber, HEARTBEAT_INTERVAL, PORT,
};
use zerocopy::{AsBytes, FromZeroes};

#[derive(Debug)]
pub struct Server {
    // configuration
    handle: SocketHandle,
    mac_addr: [u8; 6],
    bus_number: BusNumber,
    data_rate: u16,

    // state
    last_heartbeat: Instant,
    tx_start: bool,
    rx_start: bool,
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
            tx_start: false,
            rx_start: false,
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
            self.tx_start = false;
            self.rx_start = false;
            return;
        }

        if socket.can_send() {
            if now - self.last_heartbeat > HEARTBEAT_INTERVAL.into() {
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
        let packet = Packet::new_heartbeat(
            &self.mac_addr,
            &self.bus_number,
            &self.data_rate,
        );

        socket.send_slice(&packet.frame.0).map(|_| ())
    }

    /// Send can frame.
    pub fn send_frame(
        &mut self,
        sockets: &mut SocketSet,
        frame: &impl embedded_can::Frame,
    ) -> Result<(), SendError> {
        let socket = sockets.get_mut::<Socket>(self.handle);

        if !self.tx_start {
            socket.send_slice(&[0; 30])?;
            self.tx_start = true;
        }

        if let Ok(frame) = Frame::from_frame(frame) {
            socket.send_slice(&frame.0).map(|_| ())
        } else {
            Ok(())
        }
    }

    pub fn recv_frame(
        &mut self,
        sockets: &mut SocketSet,
    ) -> Result<Option<Frame>, RecvError> {
        let socket = sockets.get_mut::<Socket>(self.handle);

        if socket.can_recv() {
            if !self.rx_start {
                socket.recv_slice(&mut [0; 30]).ok();
                self.rx_start = true;
            }
        } else {
            return Ok(None);
        }

        let mut frame = Frame::new_zeroed();
        let len = socket.recv_slice(frame.as_bytes_mut())?;

        if len != size_of::<Frame>() {
            Ok(None)
        } else {
            Ok(Some(frame))
        }
    }
}
