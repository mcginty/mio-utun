extern crate mio;
extern crate mio_utun;

use std::io::{self, Write, Read};
use std::io::ErrorKind::WouldBlock;

use mio::{Poll, PollOpt, Events, Ready, Token};
use mio_utun::UtunStream;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(e) => e,
        Err(e) => panic!("{} failed with {}", stringify!($e), e),
    })
}

#[test]
fn smoke() {
    const SERVER: Token = Token(0);

    println!("connecting");
    let mut utun = UtunStream::connect("utun6").unwrap();
    let mut buf = [0u8; 1500];
    match utun.read(&mut buf) {
        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => println!("good!"),
        _ => panic!("should have WouldBlock'd")
    }
}

#[test]
fn test_server() {
    const SERVER: Token = Token(0);

    println!("connecting");
    let mut utun = UtunStream::connect("utun5").unwrap();
    let poll = Poll::new().unwrap();
    poll.register(&utun, SERVER, Ready::readable(), PollOpt::edge()).unwrap();

    let mut events = Events::with_capacity(1024);
    let mut buf = [0u8; 1500];

    println!("polling");
    poll.poll(&mut events, None).unwrap();

    println!("trying to read");
    let len = utun.read(&mut buf).unwrap();
    println!("read {} bytes!", len);
}

