use std::collections::VecDeque;

use web_sys::RtcDataChannel;

use naia_socket_shared::Ref;

use crate::{NaiaClientSocketError, Packet, PacketReceiver};

/// Handles receiving messages from the Server through a given Client Socket
#[derive(Clone, Debug)]
pub struct PacketReceiverImpl {
    data_channel: RtcDataChannel,
    dropped_outgoing_messages: Ref<VecDeque<Packet>>,
    message_queue: Ref<VecDeque<Packet>>,
}

impl PacketReceiverImpl {
    /// Create a new PacketReceiver, if supplied with the RtcDataChannel and a
    /// reference to a list of dropped messages
    pub fn new(
        data_channel: RtcDataChannel,
        dropped_outgoing_messages: Ref<VecDeque<Packet>>,
        message_queue: Ref<VecDeque<Packet>>,
    ) -> Self {
        PacketReceiverImpl {
            data_channel,
            dropped_outgoing_messages,
            message_queue,
        }
    }
}

impl PacketReceiver for PacketReceiverImpl {
    fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError> {
        match self.message_queue.borrow_mut().pop_front() {
            Some(packet) => {
                return Ok(Some(packet));
            }
            None => {
                return Ok(None);
            }
        }
    }
}