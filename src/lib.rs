//! MIO bindings for Unix Domain Sockets

#![cfg(unix)]
#![doc(html_root_url = "https://docs.rs/mio-utun/0.6")]

extern crate byteorder;
extern crate mio;
#[macro_use] extern crate nix;

#[cfg(all(target_family = "unix", any(target_os = "macos", target_os = "ios")))]
pub mod macos;
#[cfg(all(target_family = "unix", any(target_os = "macos", target_os = "ios")))]
pub use macos::UtunStream;

#[cfg(all(target_family = "unix", not(any(target_os = "macos", target_os = "ios"))))]
pub mod linux;
#[cfg(all(target_family = "unix", not(any(target_os = "macos", target_os = "ios"))))]
pub use linux::UtunStream;
