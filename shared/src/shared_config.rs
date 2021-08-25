use std::default::Default;

use super::link_conditioner_config::LinkConditionerConfig;

const DEFAULT_RTC_PATH: &str = "new_rtc_session";

/// Contains Config properties which will be shared by Server and Client sockets
#[derive(Clone, Debug)]
pub struct SocketSharedConfig {
    /// Configuration used to simulate network conditions
    pub link_condition_config: Option<LinkConditionerConfig>,
    /// The endpoint URL path to use for initiating new WebRTC sessions
    pub rtc_endpoint_path: String,
}

impl SocketSharedConfig {
    /// Creates a new SocketSharedConfig
    pub fn new(
        link_condition_config: Option<LinkConditionerConfig>,
        rtc_endpoint_path: Option<String>,
    ) -> Self {
        let endpoint_path = {
            if let Some(path) = rtc_endpoint_path {
                path
            } else {
                DEFAULT_RTC_PATH.to_string()
            }
        };

        SocketSharedConfig {
            link_condition_config,
            rtc_endpoint_path: endpoint_path,
        }
    }
}

impl Default for SocketSharedConfig {
    fn default() -> Self {
        Self {
            link_condition_config: None,
            rtc_endpoint_path: DEFAULT_RTC_PATH.to_string(),
        }
    }
}