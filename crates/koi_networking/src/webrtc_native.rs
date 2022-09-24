use std::net::SocketAddr;

use crate::*;

pub struct ReliableOrderedWebRTC {
    webrtc_unreliable_server: webrtc_unreliable::Server,
    start_instant: std::time::Instant,
    connections: Vec<Connection>,
}

struct Connection {
    remote_address: std::net::SocketAddr,
    reliable_ordered: ReliableOrdered,
}

#[derive(Clone, Copy)]
pub struct ConnectionHandle(usize);

impl ReliableOrderedWebRTC {
    pub async fn new(address: SocketAddr) -> Self {
        let webrtc_unreliable_server = webrtc_unreliable::Server::new(address, address)
            .await
            .unwrap();
        let session_endpoint = webrtc_unreliable_server.session_endpoint();

        // TODO: Create some sort of thing that listens for new connections here.
        
        Self {
            webrtc_unreliable_server,
            connections: Vec::new(),
            start_instant: std::time::Instant::now(),
        }
    }

    /// Write bytes to the outgoing stream.
    /// Nothing is sent until `flush` is called.
    pub fn send(&mut self, message: &[u8], connection_handle: ConnectionHandle) {
        let connection = &mut self.connections[connection_handle.0];
        connection.reliable_ordered.send(message);
    }

    /// Call this after multiple calls to `send`.
    pub async fn flush(&mut self, connection_handle: ConnectionHandle) {
        let connection = &mut self.connections[connection_handle.0];

        let current_time = std::time::Instant::now().duration_since(self.start_instant);
        for data in connection.reliable_ordered.flush(current_time) {
            let _ = self
                .webrtc_unreliable_server
                .send(
                    data,
                    webrtc_unreliable::MessageType::Binary,
                    &connection.remote_address,
                )
                .await;
        }
    }

    /// Reads data to `data_out`
    pub async fn read(&mut self, connection_handle: ConnectionHandle, data_out: &mut Vec<u8>) {
        let connection = &mut self.connections[connection_handle.0];

        let current_time = std::time::Instant::now().duration_since(self.start_instant);

        println!("READING HERE");
        while let Ok(message_result) = self.webrtc_unreliable_server.recv().await {
            connection
                .reliable_ordered
                .receive(current_time, &message_result.message.as_ref());
        }

        connection.reliable_ordered.read(data_out);
    }
}
