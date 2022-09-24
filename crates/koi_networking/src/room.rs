pub struct Room {
    peers: Vec<Peer>,
}

impl Room {
    pub fn new(&mut self) -> Self {
        Self { peers: Vec::new() }
    }

    pub fn peers(&mut self) -> impl Iterator<Item = &mut Peer> {
        self.peers.iter_mut()
    }

    pub fn dropped_peers(&self) {
        todo!()
    }
}

pub struct PeerHandle(usize);

pub struct Peer {
    data: Vec<u8>,
}

impl Peer {
    pub fn received(&mut self) -> &[u8] {
        &self.data
    }

    pub fn send(&mut self, bytes: &[u8]) {
        
    }
}
