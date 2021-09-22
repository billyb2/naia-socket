use std::collections::VecDeque;
use std::sync::Arc;
use std::error::Error;

use crate::Packet;
use naia_socket_shared::Ref;
use webrtc::{data::data_channel::RTCDataChannel, media::rtp::RTCPFeedback};

use hyper::body::Bytes;

/// Handles sending messages to the Server for a given Client Socket
#[derive(Clone)]
pub struct MessageSender {
    data_channel: Arc<RTCDataChannel>,
    dropped_outgoing_messages: Ref<VecDeque<Packet>>,
}

impl MessageSender {
    /// Create a new MessageSender, if supplied with the RtcDataChannel and a
    /// reference to a list of dropped messages
    pub fn new(
        data_channel: RTCDataChannel,
        dropped_outgoing_messages: Ref<VecDeque<Packet>>,
    ) -> MessageSender {
        MessageSender {
            data_channel: Arc::new(data_channel),
            dropped_outgoing_messages,
        }
    }

    /// Send a Packet to the Server
    pub async fn send(&mut self, packet: Packet) -> Result<(), Box<dyn Error + Send + Sync>> {
        if let Err(_) = self.data_channel.send(&Bytes::copy_from_slice(&packet.payload())).await {
            self.dropped_outgoing_messages
                .borrow_mut()
                .push_back(packet);
        }
        Ok(())
    }
}
