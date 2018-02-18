//! MIO bindings for Unix Domain Sockets

#![cfg(unix)]
#![doc(html_root_url = "https://docs.rs/mio-utun/0.6")]


use byteorder::{ByteOrder, NativeEndian};

use mio::unix::EventedFd;
use mio::event::Evented;
use mio::{Poll, Token, Ready, PollOpt};

use std::mem;
use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

use nix;
use nix::sys::stat::Mode;
use nix::unistd::{close, read, write};
use nix::fcntl::{open, O_RDWR, O_NONBLOCK};
use nix::sys::socket::{Shutdown, MsgFlags, shutdown, send, recv};

/// The primary class for this crate, a stream of tunneled traffic.
#[derive(Debug)]
pub struct UtunStream {
    fd: RawFd,
    name: String,
}

pub const IFNAMSIZ: usize = 16;

pub const IFF_UP:      i16 = 0x1;
pub const IFF_RUNNING: i16 = 0x40;

pub const IFF_TUN:   i16 = 0x0001;
pub const IFF_NO_PI: i16 = 0x1000;

ioctl!(write_ptr tunsetiff with b'T', 202; i32);

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
        let fd = open("/dev/net/tun", O_RDWR | O_NONBLOCK, Mode::empty())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut req = [0u8; 40]; // sizeof(struct ifreq)
        if name.len() > (IFNAMSIZ - 1) {
            return Err(io::ErrorKind::AddrNotAvailable.into())
        }

        req[..name.len()].copy_from_slice(name.as_bytes());
        NativeEndian::write_i16(&mut req[16..], IFF_TUN | IFF_NO_PI);
        let mut s = String::new();
        for &byte in req.iter() {
            s.push_str(&format!("{:02x} ", byte));
        }

        println!("{}", s);

        unsafe { tunsetiff(fd, &mut req as *mut _ as *mut _) }
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        return Ok(UtunStream {
            fd: fd,
            name: name.into(),
        })
    }

    /// Sends data on the socket to the address previously bound via connect(). On success,
    /// returns the number of bytes written.
    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        send(self.fd, buf, MsgFlags::empty())
            .map_err(|e|
                match e {
                    nix::Error::Sys(nix::Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
                    _ => io::Error::new(io::ErrorKind::Other, e)
                })
    }

    /// Receives data from the socket previously bound with connect(). On success, returns
    /// the number of bytes read and the address from whence the data came.
    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        recv(self.fd, buf, MsgFlags::empty())
            .map_err(|e|
                match e {
                    nix::Error::Sys(nix::Errno::EAGAIN) => io::ErrorKind::WouldBlock.into(),
                    _ => io::Error::new(io::ErrorKind::Other, e)
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
