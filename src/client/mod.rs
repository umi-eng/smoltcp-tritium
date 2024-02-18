//! Client socket implementations.

#[cfg(feature = "client-tcp")]
pub mod tcp;
#[cfg(feature = "client-udp")]
pub mod udp;
