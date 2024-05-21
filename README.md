# Tritium CAN Network Protocol

Rust support for the Tritium CAN network protocol. Specifically for embedded and no_std systems.

## Crates

- `tritiumcan` provides the core protocol definition, agnostic to the networking library implementation.
- `tritiumcan-smoltcp` provides a `no_std` compatible implementation using the smoltcp networking library.
