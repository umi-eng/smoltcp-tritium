//! smoltcp drivers for the Tritium CAN protocol.
//!
//! This crate provides server and client implementations for the protocol used
//! by the Tritium CAN-Ethernet adapter.

#![no_std]

pub mod tcp;
pub mod udp;

// re-export
pub use tritiumcan as proto;

use core::net::Ipv4Addr;
use smoltcp::wire::IpAddress;

// const conversion between different libray types

const BCAST_IPV4: Ipv4Addr = {
    match tritiumcan::BROADCAST {
        core::net::IpAddr::V4(addr) => addr,
        _ => unreachable!(),
    }
};

pub(crate) const BROADCAST: IpAddress = {
    let octets = BCAST_IPV4.octets();
    IpAddress::v4(octets[0], octets[1], octets[2], octets[3])
};
