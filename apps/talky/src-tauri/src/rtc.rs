use std::sync::Arc;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

pub async fn create_peer_connection() -> (Arc<RTCPeerConnection>, Arc<TrackLocalStaticSample>) {
    // Create and configure the media engine.
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs().unwrap();
    println!("Registered default codecs.");

    // Set up interceptors.
    let registry = register_default_interceptors(Registry::new(), &mut media_engine).unwrap();
    println!("Registered default interceptors.");

    // Build the API.
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    // Configure ICE servers.
    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec![
                "stun:server.loc:31899".to_owned(),
                "turn:server.loc:30665?transport=udp".to_owned(),
                "turn:server.loc:31953?transport=tcp".to_owned(),
            ],
            username: "coturn".to_string(),
            credential: "password".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Create a new RTCPeerConnection.
    let pc = api.new_peer_connection(config).await.unwrap();
    println!("Created new RTCPeerConnection.");

    // Log peer connection state changes.
    pc.on_peer_connection_state_change(Box::new(move |state| {
        println!("Peer Connection State Changed: {:?}", state);
        Box::pin(async {})
    }));

    // Create an audio track.
    let audio_track = create_audio_track();
    println!("Created audio track.");

    // Add the audio track to the peer connection.
    let rtp_sender = pc
        .add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await
        .unwrap();
    println!("Added audio track to PeerConnection.");

    // Spawn an RTCP thread to read RTCP packets.
    tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {
            // Optionally, log or process RTCP packets here.
        }
        println!("RTCP thread ended.");
    });

    (Arc::new(pc), audio_track)
}

pub async fn create_offer(pc: Arc<RTCPeerConnection>) {
    // Create the offer.
    let offer = pc.create_offer(None).await.unwrap();
    println!("Created Offer SDP:\n{}", offer.sdp);

    // Set the local description.
    pc.set_local_description(offer.clone()).await.unwrap();

    // Wait for ICE gathering to complete.
    let mut gather_complete = pc.gathering_complete_promise().await;
    let _ = gather_complete.recv().await;
    println!("ICE Gathering Complete!");

    // Log the complete local SDP.
    if let Some(local_desc) = pc.local_description().await {
        println!("Local Description after ICE gathering:\n{}", local_desc.sdp);
    } else {
        println!("Local description is missing.");
    }
}

pub fn create_audio_track() -> Arc<TrackLocalStaticSample> {
    Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: "audio/opus".to_owned(),
            ..Default::default()
        },
        "audio".to_owned(),
        "webrtc-audio".to_owned(),
    ))
}
