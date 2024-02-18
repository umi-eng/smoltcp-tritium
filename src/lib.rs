//! Tritium CAN network protocol

#![no_std]

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
