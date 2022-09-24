use crate::*;
use std::collections::VecDeque;

/// Writes a series of bytes that will be sent reliably but can arrive out of order.
/// Packets will not be received multiple times.
/// This isn't very useful on its own but can serve as the building block for other abstractions.
pub struct ReliableUnorderedStream {
    sender: ReliableUnorderedSender,
    received_start: PacketIdType,
    received: VecDeque<bool>,
    /// Keeps track of writes to the socket.
    write_actions: Vec<(usize, usize)>,
}

struct ReliableUnorderedSender {
    data_buffer_offset: usize,
    last_sent_index: usize,
    data_buffer: VecDeque<u8>,
    temp_buffer: Vec<u8>,
    packet_offset: PacketIdType,
    last_sent_packet: PacketIdType,
    packet_info: VecDeque<PacketInfo>,
    round_trip_rolling_average: core::time::Duration,
}

impl ReliableUnorderedStream {
    pub fn new() -> Self {
        Self {
            sender: ReliableUnorderedSender::new(),
            received_start: 0,
            received: VecDeque::new(),
            write_actions: Vec::new(),
        }
    }

    pub fn send(&mut self, message: &[u8]) {
        self.sender.send(message)
    }

    pub fn receive<'a>(
        &mut self,
        current_time: core::time::Duration,
        mut data: &'a [u8],
    ) -> Option<(&'a [u8], PacketIdType)> {
        let message_type: MessageType = MessageType::from_u8(*data.get(0)?)?;

        match message_type {
            MessageType::Data => {
                let packet_id = PacketIdType::from_le_bytes(
                    data.get(1..1 + std::mem::size_of::<PacketIdType>())?
                        .try_into()
                        .unwrap(),
                );

                // Check if this message was from the past, and if so ignore it.
                let id_offset = packet_id.wrapping_sub(self.received_start);
                if id_offset > PacketIdType::MAX / 2 {
                    return None;
                }

                self.received
                    .resize(self.received.len().max(id_offset as usize + 1), false);

                let packet_received = &mut self.received[id_offset as usize];
                if *packet_received {
                    return None;
                }
                *packet_received = true;

                Some((&data[1 + std::mem::size_of::<PacketIdType>()..], packet_id))
            }
            MessageType::Acknowledge => {
                while let Some(d) = data.get(1..1 + std::mem::size_of::<PacketIdType>()) {
                    let packet_id = PacketIdType::from_le_bytes(d.try_into().unwrap());
                    self.sender.acknowledge_packet(packet_id, current_time);
                    data = &data[1 + std::mem::size_of::<PacketIdType>()..];
                }
                None
            }
        }
    }

    pub fn flush(
        &mut self,
        current_time: core::time::Duration,
        // mut write_output: impl FnMut(&[u8]),
    ) -> WriteIterator {
        self.sender.flush(current_time, &mut self.write_actions);

        // Send a packet that acknowledges received packets.
        if !self.received.is_empty() {
            let message_start = self.sender.temp_buffer.len();

            //self.sender.temp_buffer.clear();
            self.sender.temp_buffer.push(MessageType::Acknowledge as _);

            // TODO: Make this respect max packet sizes.
            while self.received.front().map_or(false, |f| *f) {
                self.received.pop_front();
                self.sender
                    .temp_buffer
                    .extend_from_slice(&self.received_start.to_le_bytes());
                self.received_start = self.received_start.wrapping_add(1);
            }

            self.write_actions
                .push((message_start, self.sender.temp_buffer.len()));
        }
        WriteIterator {
            index: 0,
            reliable_unordered_stream: self,
        }
    }

    pub fn bytes_left_in_packet(&self) -> usize {
        self.sender.bytes_left_in_packet()
    }
}

pub struct WriteIterator<'a> {
    index: usize,
    reliable_unordered_stream: &'a ReliableUnorderedStream,
}

impl<'a> Iterator for WriteIterator<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        let (start, end) = *self
            .reliable_unordered_stream
            .write_actions
            .get(self.index)?;
        let result = &self.reliable_unordered_stream.sender.temp_buffer[start..end];
        self.index += 1;
        Some(result)
    }
}
/*
pub struct ReceivedIterator<'a> {
    data: &'a [u8],
}

impl<'a> Iterator for ReceivedIterator<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        println!("HERE0");
        let len = *self.data.get(0)?;
        if len == 0 {
            println!("DATA HERE: {:?}", self.data);
            return None;
        }
        println!("HERE1: {:?}", (len, self.data.len()));

        let result = self.data.get(1..1 + len as usize)?;
        println!("HERE2");

        self.data = &self.data[1 + len as usize..];
        Some(result)
    }
}
*/

struct PacketInfo {
    acknowledged: bool,
    start: usize,
    len: usize,
    id: PacketIdType,
    time_sent: core::time::Duration,
}

impl ReliableUnorderedSender {
    pub fn new() -> Self {
        Self {
            data_buffer_offset: 0,
            last_sent_index: 0,
            data_buffer: VecDeque::new(),
            temp_buffer: Vec::new(),
            packet_offset: 0,
            last_sent_packet: 0,
            packet_info: VecDeque::new(),
            round_trip_rolling_average: core::time::Duration::from_millis(100),
        }
    }

    fn send(&mut self, data: &[u8]) {
        self.data_buffer.extend(data);
    }

    pub fn acknowledge_packet(
        &mut self,
        packet_index: PacketIdType,
        current_time: core::time::Duration,
    ) {
        let packet_index = packet_index.wrapping_sub(self.packet_offset);

        // This was too far in the past to acknowledge.
        if packet_index > PacketIdType::MAX / 2 {
            return;
        }

        // This cannot be called between calls to send and flush.
        self.packet_info[packet_index as usize].acknowledged = true;
        while self.packet_info.front().map_or(false, |p| p.acknowledged) {
            let packet_info = self.packet_info.pop_front().unwrap();

            let start = packet_info.start.wrapping_sub(self.data_buffer_offset);
            self.data_buffer.drain(start..start + packet_info.len);
            self.data_buffer_offset += packet_info.len;
            self.last_sent_index -= packet_info.len;

            let round_trip_time = current_time - packet_info.time_sent;
            println!("ROUND TRIP TIME: {:?}", round_trip_time);
            println!("ROLLING AVERAGE: {:?}", self.round_trip_rolling_average);

            self.round_trip_rolling_average = self
                .round_trip_rolling_average
                .mul_f64(ROUND_TRIP_TIME_ROLLING_AVERAGE_ALPHA)
                + round_trip_time.mul_f64(1.0 - ROUND_TRIP_TIME_ROLLING_AVERAGE_ALPHA);
        }

        self.packet_offset = self.packet_offset.wrapping_add(1);
    }

    fn flush(
        &mut self,
        current_time: core::time::Duration,
        write_actions: &mut Vec<(usize, usize)>,
    ) {
        self.temp_buffer.clear();

        while self.last_sent_index != self.data_buffer.len() {
            let write_action_start = self.temp_buffer.len();

            self.temp_buffer.push(MessageType::Data as _);
            self.temp_buffer
                .extend_from_slice(&self.last_sent_packet.to_le_bytes());

            let bytes_to_write =
                PACKET_DATA_MAX_SIZE_BYTES.min(self.data_buffer.len() - self.last_sent_index);

            self.temp_buffer.extend(
                self.data_buffer
                    .range(self.last_sent_index..self.last_sent_index + bytes_to_write),
            );

            self.packet_info.push_back(PacketInfo {
                acknowledged: false,
                start: self.last_sent_index,
                len: bytes_to_write,
                id: self.last_sent_packet,
                time_sent: current_time,
            });

            self.last_sent_packet = self.last_sent_packet.wrapping_add(1);
            self.last_sent_index += bytes_to_write;

            write_actions.push((write_action_start, self.temp_buffer.len()));
        }

        // Check for packets to resend:
        // TODO: This could also resend packets when they're acknowledged out of order.
        let resend_threshold = self.round_trip_rolling_average.mul_f64(RESEND_WIGGLE_ROOM);
        for packet_info in self.packet_info.iter() {
            if !packet_info.acknowledged
                && (current_time - packet_info.time_sent) > resend_threshold
            {
                let start = packet_info.start.wrapping_sub(self.data_buffer_offset);

                // Resend the packet!
                let write_action_start = self.temp_buffer.len();
                self.temp_buffer.push(MessageType::Data as _);
                self.temp_buffer
                    .extend_from_slice(&packet_info.id.to_le_bytes());

                self.temp_buffer
                    .extend(self.data_buffer.range(start..start + packet_info.len));

                write_actions.push((write_action_start, self.temp_buffer.len()));
            }
        }
    }

    pub fn bytes_left_in_packet(&self) -> usize {
        PACKET_DATA_MAX_SIZE_BYTES - (self.data_buffer.len() % PACKET_DATA_MAX_SIZE_BYTES)
    }
}

#[test]
fn flush() {
    let mut channel_a = ReliableUnorderedStream::new();

    let mut channel_b = ReliableUnorderedStream::new();
    channel_a.send(&[0, 1, 2, 3, 4]);
    channel_a.send(&[5, 6]);

    let channel_b = &mut channel_b;
    let mut result_out: Vec<u8> = Vec::new();

    for data in channel_a.flush(core::time::Duration::from_millis(100)) {
        let (result, _id) = channel_b
            .receive(core::time::Duration::from_millis(103), data)
            .unwrap();
        result_out.extend(result);
    }

    assert_eq!(&result_out, &[0, 1, 2, 3, 4, 5, 6]);
}
