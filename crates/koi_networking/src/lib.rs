mod reliable_ordered_stream;
pub use reliable_ordered_stream::*;

mod reliable_unordered_stream;
pub use reliable_unordered_stream::*;

mod socket_trait;
pub use socket_trait::*;

#[cfg(feature = "udp")]
pub mod udp;

#[cfg(feature = "webrtc-unreliable")]
pub mod webrtc;

#[cfg(target_arch = "wasm32")]
pub mod webrtc_web;

#[cfg(feature = "room")]
pub mod room;

const PACKET_DATA_MAX_SIZE_BYTES: usize = 7;
const PACKET_HEADER_SIZE: usize = std::mem::size_of::<u32>() + std::mem::size_of::<u8>();
const TOTAL_PACKET_SIZE: usize = PACKET_DATA_MAX_SIZE_BYTES + PACKET_HEADER_SIZE;

const ROUND_TRIP_TIME_ROLLING_AVERAGE_ALPHA: f64 = 0.9;
/// How much extra a packet is allowed to be late before it's resent.
const RESEND_WIGGLE_ROOM: f64 = 0.1;

type PacketIdType = u8;

#[repr(u8)]
enum MessageType {
    Data = 0,
    Acknowledge = 1,
}

impl MessageType {
    fn from_u8(b: u8) -> Option<Self> {
        Some(match b {
            0 => Self::Data,
            1 => Self::Acknowledge,
            _ => None?,
        })
    }
}
