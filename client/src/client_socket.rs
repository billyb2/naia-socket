use std::fmt::Debug;

use naia_socket_shared::LinkConditionerConfig;

use super::{error::NaiaClientSocketError, packet::Packet};
use crate::MessageSender;

cfg_if! {
    if #[cfg(feature = "multithread")] {
        pub trait ClientSocketBaseTrait: Send + Sync {}
        impl < T > ClientSocketBaseTrait for T where T: Send + Sync {}
    } else {
        pub trait ClientSocketBaseTrait {}
        impl < T > ClientSocketBaseTrait for T {}
    }
}
/// Defines the functionality of a Naia Client Socket
pub trait ClientSocketTrait: ClientSocketBaseTrait {
    /// Receive a new packet from the socket, or a tick event
    fn receive(&mut self) -> Result<Option<Packet>, NaiaClientSocketError>;
    /// Gets a MessageSender you can use to send messages through the Server
    /// Socket
    fn get_sender(&mut self) -> MessageSender;
    /// Wraps the current socket in a LinkConditioner
    fn with_link_conditioner(
        self: Box<Self>,
        config: &LinkConditionerConfig,
    ) -> Box<dyn ClientSocketTrait>;
}
