use std::fmt::Debug;

use crossbeam::channel::Receiver;

use naia_socket_shared::{link_condition_logic, LinkConditionerConfig, TimeQueue};

use super::{error::NaiaServerSocketError, packet::Packet};

/// Used to receive packets from the Server Socket
pub trait PacketReceiverTrait: Debug {
    /// Receives a packet from the Server Socket
    fn receive(&mut self) -> Result<Option<Packet>, NaiaServerSocketError>;
}

/// Used to receive packets from the Server Socket
#[derive(Debug)]
pub struct PacketReceiver {
    channel_receiver: Receiver<Result<Packet, NaiaServerSocketError>>,
}

impl PacketReceiver {
    /// Creates a new PacketReceiver
    pub fn new(channel_receiver: Receiver<Result<Packet, NaiaServerSocketError>>) -> Self {
        PacketReceiver { channel_receiver }
    }
}

impl PacketReceiverTrait for PacketReceiver {
    fn receive(&mut self) -> Result<Option<Packet>, NaiaServerSocketError> {
        match self.channel_receiver.try_recv() {
            Ok(result) => match result {
                Ok(packet) => return Ok(Some(packet)),
                Err(_) => return Ok(None),
            },
            Err(_) => {
                return Ok(None);
            }
        }
    }
}

/// Used to receive packets from the Server Socket
#[derive(Debug)]
pub struct ConditionedPacketReceiver {
    channel_receiver: Receiver<Result<Packet, NaiaServerSocketError>>,
    link_conditioner_config: LinkConditionerConfig,
    time_queue: TimeQueue<Packet>,
}

impl ConditionedPacketReceiver {
    /// Creates a new PacketReceiver
    pub fn new(
        channel_receiver: Receiver<Result<Packet, NaiaServerSocketError>>,
        link_conditioner_config: &LinkConditionerConfig,
    ) -> Self {
        ConditionedPacketReceiver {
            channel_receiver,
            link_conditioner_config: link_conditioner_config.clone(),
            time_queue: TimeQueue::new(),
        }
    }

    fn process_packet(&mut self, packet: Packet) {
        link_condition_logic::process_packet(
            &self.link_conditioner_config,
            &mut self.time_queue,
            packet,
        );
    }

    fn has_packet(&self) -> bool {
        self.time_queue.has_item()
    }

    fn get_packet(&mut self) -> Packet {
        self.time_queue.pop_item().unwrap()
    }
}

impl PacketReceiverTrait for ConditionedPacketReceiver {
    fn receive(&mut self) -> Result<Option<Packet>, NaiaServerSocketError> {
        loop {
            match self.channel_receiver.try_recv() {
                Ok(result) => match result {
                    Err(_) => {
                        break; //TODO: Handle error here
                    }
                    Ok(packet) => {
                        self.process_packet(packet);
                    }
                },
                Err(_) => {
                    break; //TODO: Handle error here
                }
            }
        }

        if self.has_packet() {
            return Ok(Some(self.get_packet()));
        } else {
            return Ok(None);
        }
    }
}