pub trait SocketTrait {
    fn send(&self, data: &[u8]) -> std::io::Result<usize>;
    fn recv(&self, buf: &mut [u8]) -> std::io::Result<usize>;
}
