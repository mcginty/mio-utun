//! MIO bindings for Unix Domain Sockets

#![cfg(unix)]
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/mio-utun/0.6")]

extern crate iovec;
extern crate libc;
extern crate mio;
extern crate nix;

use mio::unix::EventedFd;
use mio::event::Evented;
use mio::{Poll, Token, Ready, PollOpt};

use std::mem;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

use nix::unistd::{close, read, write};
use nix::sys::socket::{AddressFamily, SockAddr, SockType, Shutdown, socket, connect, shutdown, SYSPROTO_CONTROL, SOCK_NONBLOCK};

/// The primary class for this crate, a stream of tunneled traffic.
#[derive(Debug)]
pub struct UtunStream {
    fd: RawFd,
    name: String,
}

impl UtunStream {
    /// Create a new TCP stream and issue a non-blocking connect to the
    /// specified address.
    ///
    /// This convenience method is available and uses the system's default
    /// options when creating a socket which is then connected. If fine-grained
    /// control over the creation of the socket is desired, you can use
    /// `net2::TcpBuilder` to configure a socket and then pass its socket to
    /// `TcpStream::connect_stream` to transfer ownership into mio and schedule
    /// the connect operation.
    pub fn connect(name: &str) -> io::Result<Self> {
        if &name[..4] != "utun" {
            return Err(io::ErrorKind::AddrNotAvailable.into());
        }

        let unit: u32 = name[4..].parse::<u32>()
            .map_err(|_| io::Error::from(io::ErrorKind::Other))?;

        let fd: RawFd = socket(AddressFamily::System,
                        SockType::Datagram,
                        SOCK_NONBLOCK,
                        SYSPROTO_CONTROL)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let addr = SockAddr::new_sys_control(fd,
                                             "com.apple.net.utun_control",
                                             unit + 1)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;


        connect(fd, &addr)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;


        return Ok(UtunStream {
            fd: fd,
            name: name.into(),
        })
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value (see the
    /// documentation of `Shutdown`).
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        shutdown(self.fd, how)
            .map_err(|_| io::ErrorKind::Other.into())
    }
}

impl Drop for UtunStream {
    fn drop(&mut self) {
        // Ignore error...
        let _ = close(self.fd);
    }
}

impl Read for UtunStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read(self.fd, buf)
            .map_err(|e|
                match e {
                    nix::Error::Sys(nix::Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
                    _ => io::Error::new(io::ErrorKind::Other, e)
                })
    }
}

impl<'a> Read for &'a UtunStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read(self.fd, buf)
            .map_err(|e|
                match e {
                    nix::Error::Sys(nix::Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
                    _ => io::Error::new(io::ErrorKind::Other, e)
                })
    }
}

impl Write for UtunStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write(self.fd, buf)
            .map_err(|e|
                match e {
                    nix::Error::Sys(nix::Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
                    _ => io::Error::new(io::ErrorKind::Other, e)
                })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> Write for &'a UtunStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        write(self.fd, buf)
            .map_err(|e|
                match e {
                    nix::Error::Sys(nix::Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
                    _ => io::Error::new(io::ErrorKind::Other, e)
                })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Evented for UtunStream {
    fn register(&self, poll: &Poll, token: Token,
                events: Ready, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).register(poll, token, events, opts)
    }

    fn reregister(&self, poll: &Poll, token: Token,
                  events: Ready, opts: PollOpt) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).reregister(poll, token, events, opts)
    }

    fn deregister(&self, poll: &Poll) -> io::Result<()> {
        EventedFd(&self.as_raw_fd()).deregister(poll)
    }
}

impl AsRawFd for UtunStream {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl IntoRawFd for UtunStream {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.fd;
        mem::forget(self);
        fd
    }
}
