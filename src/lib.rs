//! Tritium CAN network protocol

#![no_std]

use embedded_can::Frame;
use smoltcp::{time::Duration, wire::IpAddress};

pub(crate) mod dgram;
pub mod udp;

/// IANA port.
pub const PORT: u16 = 4876;

/// Broadcast address.
pub const BCAST_ADDR: IpAddress = IpAddress::v4(239, 255, 60, 60);

/// Protocol version identifier.
pub(crate) const PROTO_VER: u64 = 0x5472697469756;

/// Heartbeat interval.
pub const HEARTBEAT_DURATION: Duration = Duration::from_secs(1);

/// Flags bitfield.
pub(crate) struct Flags(u8);

bitflags::bitflags! {
    impl Flags: u8 {
        const Heartbeat = 1 << 7;
        const Settings = 1 << 6;
        const Remote = 1 << 1;
        const Extended = 1 << 0;
    }
}

impl Flags {
    /// Set flags from [`Frame`]
    fn from_frame(frame: &impl Frame) -> Self {
        let mut flags = Flags::empty();

        if frame.is_extended() {
            flags |= Flags::Extended
        }

        if frame.is_remote_frame() {
            flags |= Flags::Remote
        }

        flags
    }
}

/// Bus number
#[derive(Clone, Copy)]
pub struct BusNumber(u8);

impl TryFrom<u8> for BusNumber {
    type Error = ();

    /// Try create a [`BusNumber`] from a [`u8`] returning an error if the input is higher than `0xF`.
    fn try_from(value: u8) -> Result<BusNumber, Self::Error> {
        if value > 0xF {
            Err(())
        } else {
            Ok(BusNumber(value))
        }
    }
}

impl Default for BusNumber {
    /// Use the default bus number of `13`.
    fn default() -> BusNumber {
        BusNumber(13)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bus_number() {
        assert!(BusNumber::try_from(0).is_ok());
        assert!(BusNumber::try_from(13).is_ok());
        assert!(BusNumber::try_from(15).is_ok());
        assert!(BusNumber::try_from(16).is_err());
        assert!(BusNumber::try_from(255).is_err());
    }
}
