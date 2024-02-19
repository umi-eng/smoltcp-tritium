use crate::Flags;
use embedded_can::{ExtendedId, Frame, Id, StandardId};

/// Datagram length.
pub const DATAGRAM_LEN: usize = 30;

bitfield::bitfield! {
    /// Datagram used for UDP send/receive and TCP receive.
    pub struct Datagram(MSB0 [u8]);
    impl Debug;
    pub u64, version, set_version: 59, 8;
    pub u8, bus_number, set_bus_number: 63, 60;
    pub u64, client_identifier, set_client_identifier: 127, 72;
    pub u32, can_id, set_can_id: 159, 128;
    pub u8, flags, set_flags: 167, 160;
    pub u8, can_length, set_can_length: 175, 168;
    pub u64, can_data, set_can_data: 239, 176;
}

impl Datagram<[u8; DATAGRAM_LEN]> {
    pub fn new() -> Self {
        Datagram([0; DATAGRAM_LEN])
    }

    pub fn from_frame(frame: &impl Frame) -> Result<Self, ()> {
        if frame.dlc() > 8 {
            // we only support standard frames of up to 8 bytes in length.
            return Err(()); // todo: descriptive error.
        }

        let mut data: u64 = 0;

        for (n, &byte) in frame.data().iter().enumerate() {
            if n < frame.dlc() as usize {
                data |= (byte as u64) << (n * 8);
            } else {
                break;
            }
        }

        let mut dg = Datagram::new();
        dg.set_flags(Flags::from_frame(frame).bits());
        dg.set_can_id(match frame.id() {
            Id::Standard(id) => id.as_raw() as u32,
            Id::Extended(id) => id.as_raw(),
        });
        dg.set_can_length(frame.dlc() as u8);
        dg.set_can_data(data);

        Ok(dg)
    }
}

impl Frame for Datagram<[u8; DATAGRAM_LEN]> {
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

        let mut datagram = Datagram::new();
        datagram.set_can_id(id);
        datagram.set_flags(flags.bits());
        datagram.set_can_length(data.len() as u8);
        datagram.set_can_data(u64::from_be_bytes(can_data));

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

        flags.insert(Flags::Remote);

        let mut datagram = Datagram::new();
        datagram.set_can_id(id);
        datagram.set_flags(flags.bits());
        datagram.set_can_length(dlc as u8);
        datagram.set_can_data(0);

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
            Id::Extended(ExtendedId::new(self.can_id()).unwrap())
        } else {
            Id::Standard(StandardId::new(self.can_id() as u16).unwrap())
        }
    }
    fn dlc(&self) -> usize {
        self.can_length() as usize
    }

    fn data(&self) -> &[u8] {
        // todo: check if this has the right byte order
        &self.0[22..]
    }
}

pub const FRAME_DATAGRAM_LEN: usize = 14;

bitfield::bitfield! {
    /// Frame datagram only including the CAN frame section.
    ///
    /// Used for incomming frames on a TCP connection stream.
    pub struct FrameDatagram(MSB0 [u8]);
    impl Debug;
    pub u32, can_identifier, set_can_identifier: 31, 0;
    pub u8, flags, set_flags: 39, 32;
    pub u8, can_length, set_can_length: 47, 40;
    pub u64, can_data, set_can_data: 111, 48;
}

impl FrameDatagram<[u8; FRAME_DATAGRAM_LEN]> {
    pub fn new() -> Self {
        FrameDatagram([0; FRAME_DATAGRAM_LEN])
    }
}

pub const FILTER_PACKET_LEN: usize = 24;

bitfield::bitfield! {
    /// Datagram use for filt
    pub struct FilterDatagram(MSB0 [u8]);
    impl Debug;
    pub u32, fwd_identifier, set_fwd_identifier: 31, 0;
    pub u32, fwd_range, set_fwd_range: 63, 32;
    pub u8, bus_number, set_bus_bumber: 71, 64;
    pub u64, version_number, set_version_number: 123, 72;
    pub u64, client_identifier, set_client_identifier: 187, 132;
}
