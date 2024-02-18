//! Tritium CAN network protocol

#![no_std]

use smoltcp::wire::IpAddress;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

/// IANA port.
pub const PORT: u16 = 4876;

/// Broadcast address.
pub const BCAST_ADDR: IpAddress = IpAddress::v4(239, 255, 60, 60);

/// Protocol version identifier.
pub(crate) const PROTO_VER: u64 = 0x5472697469756;

/// Flags bitfield.
pub(crate) struct Flags(u8);

bitflags::bitflags! {
    impl Flags: u8 {
        const Heartbeat = 1 << 7;
        const Settings = 1 << 6;
        const FrameRtr = 1 << 1;
        const FrameExtendedId = 1 << 0;
    }
}
