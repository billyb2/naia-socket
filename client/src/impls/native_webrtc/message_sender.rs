use std::collections::VecDeque;
use std::sync::Arc;
use std::error::Error;

use crate::Packet;
use naia_socket_shared::Ref;
use webrtc::data::data_channel::RTCDataChannel;
use tokio::runtime::{Runtime, Builder};

use bytes::Bytes;

/// Handles sending messages to the Server for a given Client Socket
#[derive(Clone)]
pub struct MessageSender {
    /// The Tokio Runtime
    pub tokio_rt: Arc<Runtime>,
    data_channel: Arc<RTCDataChannel>,
    dropped_outgoing_messages: Ref<VecDeque<Packet>>,
}

impl MessageSender {
    /// Create a new MessageSender, if supplied with the RtcDataChannel and a
    /// reference to a list of dropped messages
    pub fn new(
        data_channel: Arc<RTCDataChannel>,
        dropped_outgoing_messages: Ref<VecDeque<Packet>>,
    ) -> MessageSender {
        let tokio_rt = Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

        MessageSender {
            tokio_rt: Arc::new(tokio_rt),
            data_channel: data_channel,
            dropped_outgoing_messages,
        }
    }

    /// Send a Packet to the Server
    pub fn send(&mut self, packet: Packet) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.tokio_rt.block_on(self.data_channel.send_text("Hello".to_string()));
        if let Err(e) = self.tokio_rt.block_on(self.data_channel.send(&Bytes::copy_from_slice(&packet.payload()))) {
            log::info!("Couldn't send packet {:?}", e);
            
            self.dropped_outgoing_messages
                .borrow_mut()
                .push_back(packet);
        }
        Ok(())
    }
}
