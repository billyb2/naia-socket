use std::{default::Default, net::SocketAddr};

use naia_socket_shared::SocketSharedConfig;

/// Config used to initialize a ServerSocket
#[derive(Clone, Debug)]
pub struct ServerSocketConfig {
    /// IP Address to listen on for the signaling portion of WebRTC
    pub session_listen_addr: SocketAddr,
    /// IP Address to listen on for UDP WebRTC data channels
    pub webrtc_listen_addr: SocketAddr,
    /// The public WebRTC IP address to advertise
    pub public_webrtc_addr: SocketAddr,
    /// Config which is shared between Client & Server
    pub shared: SocketSharedConfig,
}

impl ServerSocketConfig {
    /// Create a new Config which will be used to initialize a ServerSocket
    pub fn new(
        session_listen_addr: SocketAddr,
        webrtc_listen_addr: SocketAddr,
        public_webrtc_addr: SocketAddr,
        shared: SocketSharedConfig,
    ) -> Self {
        ServerSocketConfig {
            session_listen_addr,
            webrtc_listen_addr,
            public_webrtc_addr,
            shared,
        }
    }
}

impl Default for ServerSocketConfig {
    fn default() -> Self {
        ServerSocketConfig::new(
            "127.0.0.1:14191"
                .parse()
                .expect("could not parse HTTP address/port"),
            "127.0.0.1:14192"
                .parse()
                .expect("could not parse WebRTC data address/port"),
            "127.0.0.1:14192"
                .parse()
                .expect("could not parse advertised public WebRTC data address/port"),
            SocketSharedConfig::default(),
        )
    }
}