//! MIO bindings for Unix Domain Sockets

#![cfg(unix)]
#![doc(html_root_url = "https://docs.rs/mio-utun/0.6")]

#[macro_use] extern crate nix;
extern crate mio;


#[cfg(all(unix, any(target_os = "macos", target_os = "ios")))]
pub mod macos;
#[cfg(all(unix, any(target_os = "macos", target_os = "ios")))]
pub use macos::UtunStream;

#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
extern crate byteorder;
#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
pub mod linux;
#[cfg(all(unix, not(any(target_os = "macos", target_os = "ios"))))]
pub use linux::UtunStream;
