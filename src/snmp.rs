use std::net::{IpAddr, ToSocketAddrs};
use crate::Error;

pub struct PTouchPrinter<D> {
    pub interface: D,
    pub ip_addr: Option<IpAddr>,
    send_buffer: Option<Vec<u8>>, // Probably use a type (of PTouchPrinter) to diff between buffered and direct io
}

impl PTouchPrinter<PTouchTcpInterface> {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        from_addr(addr)
    }
}

pub fn from_addr<A: ToSocketAddrs>(addr: A) -> Result<PTouchPrinter<PTouchTcpInterface>> {
    let ip_addr = addr
        .to_socket_addrs()
        .ok()
        .and_then(|mut e| e.next())
        .map(|sa| sa.ip());

    Ok(PTouchPrinter {
        ip_addr,
        interface: PTouchTcpInterface::new(addr, Some(DEFAULT_TIMEOUT))?,
        // send_buffer: Some(Vec::with_capacity(2048)), // buffered IO
        send_buffer: None, // unbuffered, immediate IO
    })
}

impl<D: PTouchInterface> PTouchPrinter<D> {
    // pub fn get_status(&mut self) -> Result<Status> {
    //     Ok(Status)
    // }

    // Todo: send `Command` type
    pub fn write(&mut self, data: impl AsRef<[u8]>) -> Result<(), crate::Error::SNMPError> {
        if let Some(buffer) = self.send_buffer.as_mut() {
            buffer.extend_from_slice(data.as_ref());
            Ok(())
        } else {
            self.interface.write(data.as_ref())
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        if let Some(buffer) = self.send_buffer.as_mut() {
            self.interface.write(buffer.as_slice())?;
            buffer.clear();
        }

        Ok(())
    }

    // // Todo: return `Response` type
    // pub fn send_raw_with_response(&mut self, command: impl AsRef<[u8]>) -> Result<Vec<u8>> {
    //     self.interface.write(command.as_ref())?;
    //     self.interface.read_vec()
    // }
}

use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

pub trait PTouchInterface: Sized {
    fn name(&self) -> String;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;

    fn read_vec(&mut self) -> Result<Vec<u8>>;

    fn write(&mut self, data: &[u8]) -> Result<()>;

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct PTouchTcpInterface {
    socket: TcpStream,
}

impl PTouchTcpInterface {
    pub fn new<A: ToSocketAddrs>(addr: A, read_timeout: Option<Duration>) -> Result<Self> {
        let socket = TcpStream::connect(addr)?;
        socket.set_read_timeout(read_timeout)?;

        println!("sokket: {socket:?}");
        Ok(PTouchTcpInterface { socket })
    }
}

impl PTouchInterface for PTouchTcpInterface {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Ok(self.socket.read(buf)?)
    }

    fn name(&self) -> String {
        format!(
            "PTouch TCP interface on {}",
            self.socket
                .peer_addr()
                .map(|sa| sa.to_string())
                .unwrap_or_default()
        )
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        self.socket.write_all(data)?;
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        self.socket.flush()?;
        Ok(())
    }

    fn read_vec(&mut self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.socket.read_to_end(&mut buf)?;
        Ok(buf)
    }
}
