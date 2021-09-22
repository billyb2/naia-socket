extern crate log;
use log::info;
use webrtc::peer::ice::ice_candidate::RTCIceCandidate;

//TODO: Parking Lot Mutex
use std::sync::Arc;
use std::time::Duration;
use std::{collections::VecDeque, net::SocketAddr};

use crate::{error::NaiaClientSocketError, Packet};

use naia_socket_shared::Ref;

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
use tokio::sync::Mutex;
use hyper::service::{make_service_fn, service_fn};


use hyper::{Body, Client, Method, Request, Response, Server, StatusCode};


#[allow(unused_must_use)]
pub async fn webrtc_initialize(
    socket_address: SocketAddr,
    msg_queue: Ref<VecDeque<Result<Option<Packet>, NaiaClientSocketError>>>,
) -> RTCDataChannel {
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
            println!("Peer Connection State has changed: {}", s);

            if s == RTCPeerConnectionState::Failed {
                // Wait until PeerConnection has had no network activity for 30 seconds or another failure. It may be reconnected using an ICE Restart.
                // Use webrtc.PeerConnectionStateDisconnected if you are interested in detecting faster timeout.
                // Note that the PeerConnection may come back from PeerConnectionStateDisconnected.
                println!("Peer Connection has gone to failed exiting");
                std::process::exit(0);
            }

            Box::pin(async {})
        }))
        .await;

    let d1 = Arc::clone(&data_channel);
    data_channel.on_open(Box::new(move || {
        println!("Data channel '{}'-'{}' open. Random messages will now be sent to any connected DataChannels every 5 seconds", d1.label(), d1.id());

        let d2 = Arc::clone(&d1);
        Box::pin(async {})
    })).await;


    data_channel
        .on_message(Box::new(move |msg: DataChannelMessage| {
            msg_queue.borrow_mut().push_back(Ok(Some(
                Packet::new_raw(msg.data.as_ref().into()
            ))));
            Box::pin(async {})
        }))
        .await;

    let PENDING_CANDIDATES: Arc<Mutex<Vec<RTCIceCandidate>>> = Arc::new(Mutex::new(vec![]));

     // When an ICE candidate is available send to the other Pion instance
    // the other Pion instance will add this candidate by calling AddICECandidate
    let peer_connection2 = Arc::clone(&peer_conn);
    let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
    let addr2 = socket_address.clone();

    let offer = peer_conn.create_offer(None).await.unwrap();

    // Send our offer to the HTTP server listening in the other process
    let payload = match serde_json::to_string(&offer) {
        Ok(p) => p,
        Err(err) => panic!("{}", err),
    };


    let req = match Request::builder()
            .method(Method::POST)
            .uri(server_url_str)
            .header("content-type", "application/json; charset=utf-8")
            .body(Body::from(payload.clone()))
        {
            Ok(req) => req,
            Err(err) => panic!("{}", err),
        };

    let resp = match Client::new().request(req).await {
        Ok(resp) => resp,
        Err(err) => {
            panic!("{}", err);
        }
    };
    let response_string: String = payload;

    peer_conn.set_local_description(offer).await.unwrap();

    let peer_connection2 = Arc::clone(&peer_conn);
    let pending_candidates2 = Arc::clone(&PENDING_CANDIDATES);
    peer_conn
        .on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
            //println!("on_ice_candidate {:?}", c);

            let peer_connection3 = Arc::clone(&peer_connection2);
            let pending_candidates3 = Arc::clone(&pending_candidates2);
            let addr3 = addr2.clone();
            Box::pin(async move {
                let c = c.unwrap();
                let desc = peer_connection3.remote_description().await;
                if desc.is_none() {
                    let mut cs = pending_candidates3.lock().await;
                    cs.push(c);
                }
            })
        }))
        .await;


    if let Ok(data_channel) = Arc::try_unwrap(data_channel) {
        data_channel

    } else {
        panic!("Error!");

    }

}
