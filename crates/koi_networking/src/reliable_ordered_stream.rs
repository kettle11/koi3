use std::collections::VecDeque;

use crate::*;

pub struct ReliableOrderedWithSocket<Socket: SocketTrait> {
    reliable_ordered: ReliableOrdered,
    socket: Socket,
    temp_buffer: Vec<u8>,
    start_instant: std::time::Instant,
}

impl<Socket: SocketTrait> ReliableOrderedWithSocket<Socket> {
    pub fn new(socket: Socket) -> Self {
        Self {
            reliable_ordered: ReliableOrdered::new(),
            socket,
            temp_buffer: vec![0; TOTAL_PACKET_SIZE],
            start_instant: std::time::Instant::now(),
        }
    }

    /// Write bytes to the outgoing stream.
    pub fn send(&mut self, message: &[u8]) {
        self.reliable_ordered.send(message);
    }

    pub fn flush(&mut self) {
        let current_time = std::time::Instant::now().duration_since(self.start_instant);
        for data in self.reliable_ordered.flush(current_time) {
            println!("SENDING DATA: {:?}", data);
            self.socket.send(data).unwrap();
        }
    }

    /// Reads data to `data_out`
    /// Will block if the underlying socket is blocking.
    pub fn read(&mut self, data_out: &mut Vec<u8>) {
        let current_time = std::time::Instant::now().duration_since(self.start_instant);

        println!("READING HERE");
        while let Ok(bytes_written) = self.socket.recv(&mut self.temp_buffer) {
            println!("RECEIVING DATA: {:?}", bytes_written);
            self.reliable_ordered
                .receive(current_time, &self.temp_buffer[0..bytes_written]);
        }

        self.reliable_ordered.read(data_out);
    }
}

pub struct ReliableOrdered {
    reliable_unordered: ReliableUnorderedStream,
    data_in: VecDeque<u8>,
    next_expected_packet: PacketIdType,
    future_packets: VecDeque<Option<(usize, [u8; PACKET_DATA_MAX_SIZE_BYTES])>>,
}

impl ReliableOrdered {
    pub fn new() -> Self {
        Self {
            reliable_unordered: ReliableUnorderedStream::new(),
            data_in: VecDeque::new(),
            next_expected_packet: 0,
            future_packets: VecDeque::new(),
        }
    }

    /// Write bytes to the outgoing stream.
    pub fn send(&mut self, message: &[u8]) {
        // TODO: I think this needs to account for the header.
        for chunk in message.chunks(255) {
            self.reliable_unordered.send(chunk);
        }
    }

    /// Receives incoming bytes.
    pub fn receive(&mut self, current_time: core::time::Duration, data: &[u8]) {
        println!("DATA HERE 0: {:?}", data);
        if let Some((data_in, packet_id)) = self.reliable_unordered.receive(current_time, data) {
            println!("PACKET ID: {:?}", packet_id);
            if packet_id == self.next_expected_packet {
                self.data_in.extend(data_in);
                self.next_expected_packet = self.next_expected_packet.wrapping_add(1);

                while self.future_packets.front().map_or(false, |f| f.is_some()) {
                    let (len, data) = self.future_packets.pop_front().unwrap().unwrap();
                    self.data_in.extend(&data[..len]);
                    self.next_expected_packet = self.next_expected_packet.wrapping_add(1);
                }
            } else {
                let mut bytes = [0; PACKET_DATA_MAX_SIZE_BYTES];
                bytes[0..data_in.len()].copy_from_slice(data_in);

                let offset = packet_id.wrapping_sub(self.next_expected_packet) as usize;
                self.future_packets
                    .resize(self.future_packets.len().max(offset), None);
                self.future_packets[offset - 1] = Some((data_in.len(), bytes));
            }
        }
    }

    /// Reads internal buffer to `data_out` then erases the internal buffer.
    pub fn read(&mut self, data_out: &mut Vec<u8>) {
        data_out.extend(self.data_in.drain(..))
    }

    /// Sends all enqued outgoing messages and acknowledgements.
    /// Needs to be called after calls to `receive`
    pub fn flush(&mut self, current_time: core::time::Duration) -> WriteIterator {
        self.reliable_unordered.flush(current_time)
    }
}

#[test]
fn udp() {
    let client_address = "127.0.0.1:8080";
    let server_address = "127.0.0.1:8081";
    let mut client = ReliableOrderedWithSocket::new_udp(client_address, server_address).unwrap();
    let mut server = ReliableOrderedWithSocket::new_udp(server_address, client_address).unwrap();

    client.send(&[2, 3, 4]);
    client.flush();

    let mut data_out = Vec::new();

    while data_out.is_empty() {
        server.read(&mut data_out);
    }

    assert_eq!(&data_out, &[2, 3, 4]);
}

#[test]
fn reliable_ordered() {
    let mut channel_a = ReliableOrdered::new();
    let mut channel_b = ReliableOrdered::new();

    channel_a.send(&[0, 1, 2, 3, 4]);
    channel_a.send(&[0, 1]);

    let channel_b = &mut channel_b;
    for data in channel_a.flush(core::time::Duration::from_millis(100)) {
        channel_b.receive(core::time::Duration::from_millis(103), data);
    }

    let mut data_out = Vec::new();
    channel_b.read(&mut data_out);
    println!("DATA OUT: {:?}", data_out);
    assert_eq!(&data_out, &[0, 1, 2, 3, 4, 0, 1])
}

#[test]
fn reliable_ordered_out_of_order() {
    let mut channel_a = ReliableOrdered::new();
    let mut channel_b = ReliableOrdered::new();

    channel_a.send(&[0, 1, 2, 3, 4, 5, 6, 7]);
    channel_a.send(&[8, 9]);

    let channel_b = &mut channel_b;
    let mut packets: Vec<Vec<u8>> = Vec::new();

    for data in channel_a.flush(core::time::Duration::from_millis(100)) {
        packets.push(data.into());
    }

    println!("PACKETS LEN: {:?}", packets.len());
    println!("PACKETS: {:?}", packets);
    for packet in packets.iter().rev() {
        channel_b.receive(core::time::Duration::from_millis(103), &packet);
    }

    let mut data_out = Vec::new();
    channel_b.read(&mut data_out);
    println!("DATA OUT: {:?}", data_out);
    assert_eq!(&data_out, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
}

#[test]
fn reliable_ordered_duplicates() {
    let mut channel_a = ReliableOrdered::new();
    let mut channel_b = ReliableOrdered::new();

    channel_a.send(&[0, 1, 2, 3, 4, 5, 6, 7]);
    channel_a.send(&[8, 9]);

    let channel_b = &mut channel_b;
    let mut packets: Vec<Vec<u8>> = Vec::new();

    for data in channel_a.flush(core::time::Duration::from_millis(100)) {
        packets.push(data.into());
    }

    println!("PACKETS LEN: {:?}", packets.len());
    println!("PACKETS: {:?}", packets);
    for packet in packets.iter() {
        channel_b.receive(core::time::Duration::from_millis(103), &packet);
        channel_b.receive(core::time::Duration::from_millis(108), &packet);
    }

    let mut data_out = Vec::new();
    channel_b.read(&mut data_out);
    let _ = channel_b.flush(core::time::Duration::from_millis(110));

    channel_b.receive(core::time::Duration::from_millis(120), &packets[0]);
    channel_b.read(&mut data_out);

    println!("DATA OUT: {:?}", data_out);
    assert_eq!(&data_out, &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
}

#[test]
fn id_wrap() {
    let mut channel_a = ReliableOrdered::new();
    let mut channel_b = ReliableOrdered::new();

    let mut data = Vec::new();
    let mut count: u8 = 0;
    for _ in 0..(PACKET_DATA_MAX_SIZE_BYTES * 129) {
        data.push(count);
        count = count.wrapping_add(1);
    }

    channel_a.send(&data);

    let channel_b = &mut channel_b;
    for data in channel_a.flush(core::time::Duration::from_millis(100)) {
        channel_b.receive(core::time::Duration::from_millis(103), data);
    }

    let mut data_out = Vec::new();
    channel_b.read(&mut data_out);
    // println!("DATA OUT: {:?}", data_out);
    println!("LEFT LEN: {:?}", data_out.len());
    println!("RIGHT LEN: {:?}", data.len());
    // assert_eq!(&data_out, &data)
}
