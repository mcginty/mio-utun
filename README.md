# mio-utun

[Documentation](https://docs.rs/mio-utun)

This crate is a mio wrapper for the utun interface (userspace tunnel) used by macOS.

[mio]: https://github.com/carllerche/mio

```toml
# Cargo.toml
[dependencies]
mio-utun = "0.6"
mio = "0.6"
```

## Notice

This is a work-in-progress with documentation and further testing and features a TODO.

## Usage

There is only one export, `UtunStream`, and it behaves similarly to the standard
`TcpStream`.


# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

