use crate::*;

impl SocketTrait for std::net::UdpSocket {
    fn send(&self, data: &[u8]) -> std::io::Result<usize> {
        self.send(data)
    }

    fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.recv(buf)
    }
}

impl ReliableOrderedWithSocket<std::net::UdpSocket> {
    pub fn new_udp(local_address: &str, remote_address: &str) -> Result<Self, std::io::Error> {
        use std::str::FromStr;
        let local_address = std::net::SocketAddr::from_str(local_address).unwrap();
        let socket = std::net::UdpSocket::bind(local_address)?;
        socket.set_nonblocking(true).unwrap();
        socket.connect(remote_address)?;
        Ok(Self::new(socket))
    }
}
