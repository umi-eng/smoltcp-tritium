use crate::{BusNumber, Flags, PROTOCOL_VERSION};
use embedded_can::{ExtendedId, Id, StandardId};
use zerocopy::{AsBytes, FromBytes, FromZeroes};

/// Datagram header length.
const HEADER_LEN: usize = 16;

bitfield::bitfield! {
    /// Datagram header, used when receiving UDP data and sending TCP data.
    #[derive(AsBytes, FromBytes, FromZeroes)]
    #[repr(transparent)]
    #[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
    pub struct HeaderBitfield(MSB0 [u8]);
    impl Debug;
    pub u64, version, set_version: 59, 8;
    pub u8, bus_number, set_bus_number: 63, 60;
    pub u64, client_identifier, set_client_identifier: 127, 72;
}

pub type Header = HeaderBitfield<[u8; HEADER_LEN]>;

impl Header {
    pub fn new() -> Self {
        HeaderBitfield([0; HEADER_LEN])
    }
}

impl embedded_can::Frame for Frame {
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
        if data.len() > 8 {
            return None;
        }

        let (flags, id) = match id.into() {
            Id::Standard(id) => (Flags::empty(), id.as_raw() as u32),
            Id::Extended(id) => (Flags::Extended, id.as_raw()),
        };

        let mut can_data = [0u8; 8];
        can_data[..data.len()].copy_from_slice(data);

        let mut datagram = Frame::new();
        datagram.set_id(id);
        datagram.set_flags(flags.bits());
        datagram.set_dlc(data.len() as u8);
        datagram.set_data(u64::from_be_bytes(can_data));

        Some(datagram)
    }

    fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
        if dlc > 8 {
            return None;
        }

        let (mut flags, id) = match id.into() {
            Id::Standard(id) => (Flags::empty(), id.as_raw() as u32),
            Id::Extended(id) => (Flags::Extended, id.as_raw()),
        };

        flags |= Flags::Remote;

        let mut datagram = Frame::new();
        datagram.set_id(id);
        datagram.set_flags(flags.bits());
        datagram.set_dlc(dlc as u8);
        datagram.set_data(0);

        Some(datagram)
    }

    fn is_extended(&self) -> bool {
        Flags::from_bits(self.flags())
            .unwrap()
            .intersects(Flags::Extended)
    }

    fn is_remote_frame(&self) -> bool {
        Flags::from_bits(self.flags())
            .unwrap()
            .intersects(Flags::Remote)
    }

    fn id(&self) -> Id {
        if self.is_extended() {
            Id::Extended(ExtendedId::new(self.id()).unwrap())
        } else {
            Id::Standard(StandardId::new(self.id() as u16).unwrap())
        }
    }
    fn dlc(&self) -> usize {
        self.dlc() as usize
    }

    fn data(&self) -> &[u8] {
        // todo: check if this has the right byte order
        &self.0[22..]
    }
}

pub const FRAME_LEN: usize = 14;

bitfield::bitfield! {
    /// Frame datagram only including the CAN frame section.
    ///
    /// Used for incomming frames on a TCP connection stream.
    #[derive(AsBytes, FromBytes, FromZeroes)]
    #[repr(transparent)]
    #[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
    pub struct FrameBitfield(MSB0 [u8]);
    impl Debug;
    impl FromZeroes;
    impl FromBytes;
    impl AsBytes;
    pub u32, id, set_id: 31, 0;
    pub u8, flags, set_flags: 39, 32;
    pub u8, dlc, set_dlc: 47, 40;
    pub u64, data, set_data: 111, 48;
}

pub type Frame = FrameBitfield<[u8; FRAME_LEN]>;

impl Frame {
    pub fn new() -> Self {
        FrameBitfield([0; FRAME_LEN])
    }

    pub fn from_frame(frame: &impl embedded_can::Frame) -> Result<Self, ()> {
        if frame.dlc() > 8 {
            // we only support standard frames of up to 8 bytes in length.
            return Err(()); // todo: descriptive error.
        }

        let mut data: u64 = 0;

        for (n, &byte) in frame.data().iter().rev().enumerate() {
            if n < frame.dlc() as usize {
                data |= (byte as u64) << (n * 8);
            } else {
                break;
            }
        }

        let mut dg = Frame::new();
        dg.set_flags(Flags::from_frame(frame).bits());
        dg.set_id(match frame.id() {
            Id::Standard(id) => id.as_raw() as u32,
            Id::Extended(id) => id.as_raw(),
        });
        dg.set_dlc(frame.dlc() as u8);
        dg.set_data(data);

        Ok(dg)
    }
}

/// Complete datagram packet.
///
/// Used when receiving UDP frames and sending frames for both UDP and TCP.
#[derive(Debug, FromBytes, AsBytes, FromZeroes)]
#[repr(C)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct Packet {
    pub header: Header,
    pub frame: Frame,
}

impl Packet {
    pub fn new_heartbeat(
        mac_addr: &[u8; 6],
        bus_number: &BusNumber,
        data_rate: &u16,
    ) -> Self {
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
        packet.header.set_version(PROTOCOL_VERSION);
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

    pub fn as_bytes(&self) -> &[u8] {
        // is safe because we use size_of::<Packet>
        unsafe {
            ::core::slice::from_raw_parts(
                self as *const _ as *const u8,
                ::core::mem::size_of::<Packet>(),
            )
        }
    }
}

/// Filter setting datagram length.
pub const FILTER_LEN: usize = 24;

bitfield::bitfield! {
    /// Datagram use for filt
    #[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
    pub struct FilterBitfield(MSB0 [u8]);
    impl Debug;
    pub u32, fwd_identifier, set_fwd_identifier: 31, 0;
    pub u32, fwd_range, set_fwd_range: 63, 32;
    pub u8, bus_number, set_bus_bumber: 71, 64;
    pub u64, version_number, set_version_number: 123, 72;
    pub u64, client_identifier, set_client_identifier: 187, 132;
}

pub type Filter = FilterBitfield<[u8; FILTER_LEN]>;

impl Filter {
    pub fn new() -> Self {
        FilterBitfield([0; FILTER_LEN])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::mem::{size_of, size_of_val};

    #[test]
    fn header_type_length() {
        assert_eq!(size_of_val(&Header::new()), 16)
    }

    #[test]
    fn frame_type_length() {
        assert_eq!(size_of_val(&Frame::new()), 14)
    }

    #[test]
    fn packet_type_length() {
        assert_eq!(size_of::<Packet>(), 30)
    }
}
