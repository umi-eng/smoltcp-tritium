# Tritium CAN network protocol

Implemented in [`smoltcp`](https://github.com/smoltcp-rs/smoltcp) for embedded and `no_std` environments.

## Usage

Add an entry to your `Cargo.toml`:

```toml
[dependencies]
smoltcp-tritium = "0.1.0"
```

## Future work

- Async support.
  - Waiting on `embedded-can` async support.
  - May need to use `rtic_sync` or `embassy-sync` to mutably share the `SocketSet`.
- Common trait(s) for UDP/TCP allowing for generic code using either.

## Minimum supported Rust version

There will not yet be any guarantees for the minimum supported Rust version until this crate reaches maturity.
