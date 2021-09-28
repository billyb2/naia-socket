extern crate log;
use log::info;
use webrtc::peer::ice::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};

use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::VecDeque, net::SocketAddr};

use crate::{error::NaiaClientSocketError, Packet};

use naia_socket_shared::Ref;

use serde_json::Value;

use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data::data_channel::RTCDataChannel;
use webrtc::peer::configuration::RTCConfiguration;
use webrtc::peer::ice::ice_server::RTCIceServer;
use webrtc::peer::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer::sdp::session_description::RTCSessionDescription;
use webrtc::util::math_rand_alpha;
use webrtc::peer::sdp::sdp_type::RTCSdpType;

use tokio::sync::Mutex;

use reqwest::Client;


#[allow(unused_must_use)]
pub async fn webrtc_initialize(
    socket_address: SocketAddr,
    msg_queue: Ref<VecDeque<Result<Option<Packet>, NaiaClientSocketError>>>,
) -> Arc<RTCDataChannel> {
    let server_url_str = format!("http://{}/new_rtc_session", socket_address);

    // Create a MediaEngine object to configure the supported codec
    let mut m = MediaEngine::default();

    // Register default codecs
    m.register_default_codecs().unwrap();

    // Create the API object with the MediaEngine
    let api = APIBuilder::new()
        .with_media_engine(m)
        .build();

    // Prepare the configuration
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_owned()],
            ..Default::default()
        }],
        ..Default::default()

    };

    let peer_conn = Arc::new(api.new_peer_connection(config).await.unwrap());
    let data_channel = peer_conn.create_data_channel("data", None).await.unwrap();

    peer_conn
        .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            info!("Peer Connection State has changed: {}", s);

            if s == RTCPeerConnectionState::Failed {
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                info!("Peer Connection has gone to failed exiting");
                std::process::exit(0);
            }

            Box::pin(async {})
        }))
        .await;

    data_channel.on_open(Box::new(move || {
        Box::pin(async {})
    })).await;


    data_channel
        .on_message(Box::new(move |msg: DataChannelMessage| {
            msg_queue.borrow_mut().push_back(Ok(Some(
                Packet::new_raw(msg.data.as_ref().into()
            ))));
            info!("Message: {:?}", msg);
            Box::pin(async {})
        }))
        .await;

    let PENDING_CANDIDATES: Arc<Mutex<Vec<RTCIceCandidate>>> = Arc::new(Mutex::new(vec![]));

    //let mut candidate_init_dict: RtcIceCandidateInit = RtcIceCandidateInit::new(session_response.candidate.candidate.as_str());

     // When an ICE candidate is available send to the other Pion instance
    // the other Pion instance will add this candidate by calling AddICECandidate
    let peer_connection2 = Arc::clone(&peer_conn);
    let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
    let addr2 = socket_address.clone();

    let offer = peer_conn.create_offer(None).await.unwrap();
    peer_conn.set_local_description(offer.clone()).await.unwrap();

    // Send our offer to the HTTP server listening in the other process    
    let client = Client::new();

    let req = client.post(server_url_str).header("content-type", "application/json").header("accept", "application/json, text/plain, */*").body(offer.sdp.clone());

    let resp = match req.send().await {
        Ok(resp) => resp,
        Err(err) => {
            panic!("Could not send request, original error: {:?}", err);
        }
    };
    let mut response_string = resp.text().await.unwrap();

    let json_resp = serde_json::from_str::<Value>(&response_string).unwrap();
    let answer_json_str = json_resp["answer"].to_string();
    let ice_candidate_json_str = json_resp["candidate"].to_string();

    info!("\n\n\n{:?}\n\n\n", &json_resp["candidate"]["candidate"].as_str().unwrap().to_owned());

    let answer = serde_json::from_str::<RTCSessionDescription>(&answer_json_str).unwrap();
    let ice_candidate = RTCIceCandidateInit {
        candidate: json_resp["candidate"]["candidate"].as_str().unwrap().to_owned(),
        sdp_mid: json_resp["candidate"]["sdpMid"].as_str().unwrap().to_owned(),
        sdp_mline_index: u16::try_from(json_resp["candidate"]["sdpMLineIndex"].as_u64().unwrap()).ok().unwrap(),
        ..Default::default()
        
    };

    peer_conn.set_remote_description(answer).await.unwrap();

    if let Err(e) = peer_conn.add_ice_candidate(ice_candidate).await {
        panic!("Error on adding ice candidate: {:?}", e);

    };

    let peer_connection2 = Arc::clone(&peer_conn);
    let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
    peer_conn
        .on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
            let peer_connection3 = Arc::clone(&peer_connection2);
            let pending_candidates3 = Arc::clone(&pending_candidates2);
            let addr3 = addr2.clone();
            Box::pin(async move {
                if let Some(c) = c {
                    info!("ICE \n\n\n\n\n\n\nCANDIDATE!!!!!!! {}", serde_json::to_string(&c).unwrap());

                    let desc = peer_connection3.remote_description().await;
                    if desc.is_none() {
                        let mut cs = pending_candidates3.lock().await;
                        cs.push(c);
                    }
                }
            })
        }))
        .await;


    data_channel

}
