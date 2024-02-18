//! Tritium CAN network protocol

#![no_std]

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

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
