//! MIO bindings for Unix Domain Sockets

#![cfg(unix)]
#![doc(html_root_url = "https://docs.rs/mio-utun/0.6")]

extern crate byteorder;
extern crate mio;
#[macro_use] extern crate nix;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::UtunStream;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::UtunStream;
