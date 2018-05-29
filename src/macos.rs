//! MIO bindings for Unix Domain Sockets

#![cfg(unix)]
#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/mio-utun/0.6")]

extern crate mio;
extern crate nix;

use mio::unix::EventedFd;
use mio::event::Evented;
use mio::{Poll, Token, Ready, PollOpt};

use nix::errno::Errno;
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::unistd::{close, read, write};
use nix::sys::socket::{AddressFamily, SockAddr, SockType, SockFlag, SockProtocol, Shutdown, socket, connect, shutdown};

use std::mem;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd, FromRawFd};

/// The primary class for this crate, a stream of tunneled traffic.
#[derive(Debug)]
pub struct UtunStream {
    fd: RawFd,
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

        let unit: u32 = if name.len() == 4 {
            0
        } else {
            1 + name[4..].parse::<u32>().map_err(|_| io::Error::from(io::ErrorKind::Other))?
        };

        let fd: RawFd = socket(AddressFamily::System,
                               SockType::Datagram,
                               SockFlag::empty(),
                               SockProtocol::KextControl)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let addr = SockAddr::new_sys_control(fd,
                                             "com.apple.net.utun_control",
                                             unit)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        fcntl(fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        connect(fd, &addr)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;


        return Ok(UtunStream { fd })
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
                    nix::Error::Sys(Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
                    _ => io::Error::new(io::ErrorKind::Other, e)
                })
    }
}

impl<'a> Read for &'a UtunStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        read(self.fd, buf)
            .map_err(|e|
                match e {
                    nix::Error::Sys(Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
                    _ => io::Error::new(io::ErrorKind::Other, e)
                })
    }
}

impl Write for UtunStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        match buf[0] >> 4 {
            4 => write(self.fd, &[&[0u8, 0x00, 0x00, 0x02], buf].concat()),
            6 => write(self.fd, &[&[0u8, 0x00, 0x00, 0x1e], buf].concat()),
            _ => return Err(io::Error::new(io::ErrorKind::Other, "unrecognized IP version")),
        }.map(|len| len - 4)
        .map_err(|e| match e {
            nix::Error::Sys(Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
            _ => io::Error::new(io::ErrorKind::Other, e)
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> Write for &'a UtunStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() == 0 {
            return Ok(0);
        }

        match buf[0] >> 4 {
            4 => write(self.fd, &[&[0u8, 0x00, 0x00, 0x02], buf].concat()),
            6 => write(self.fd, &[&[0u8, 0x00, 0x00, 0x1e], buf].concat()),
            _ => return Err(io::Error::new(io::ErrorKind::Other, "unrecognized IP version")),
        }.map(|len| len - 4)
        .map_err(|e| match e {
            nix::Error::Sys(Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
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

impl FromRawFd for UtunStream {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self { fd }
    }
}
