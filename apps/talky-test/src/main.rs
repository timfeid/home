use bytemuck::cast_slice;
use cpal::{BufferSize, SampleRate, StreamConfig};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep as async_sleep;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tungstenite::Bytes;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use opus::{Application, Channels, Encoder};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_OPUS};
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::{
    RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType,
};
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

const JOIN_CODE: &str = "room1";

pub fn configure_input_stream(device: &cpal::Device) -> StreamConfig {
    let supported_config = device
        .default_input_config()
        .expect("Failed to get default input config");
    let mut config: StreamConfig = supported_config.into();
    config.buffer_size = BufferSize::Fixed(1024);
    config.sample_rate = SampleRate(48000);
    config.channels = 2;
    println!("Configured input stream: {:?}", config);
    config
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting WebRTC Audio Test Client");

    let (ws_stream, _) = connect_async("ws://localhost:8080/soundhouse").await?;
    println!("Connected to soundhouse server");

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));

    let init_msg = json!({
        "join_code": JOIN_CODE,
        "role": "offerer"
    })
    .to_string();
    println!("Sending init message: {}", init_msg);
    ws_sender
        .lock()
        .await
        .send(Message::Text(init_msg.into()))
        .await?;

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

    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;

    let registry = register_default_interceptors(Registry::new(), &mut media_engine)?;
    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .build();

    let peer_connection = Arc::new(api.new_peer_connection(config).await?);
    println!("Created PeerConnection");

    let pc_clone = Arc::clone(&peer_connection);
    peer_connection.on_peer_connection_state_change(Box::new(move |state| {
        println!("Peer Connection State Changed: {:?}", state);
        if state == RTCPeerConnectionState::Failed {
            println!("Peer connection failed, attempting restart...");
        }
        Box::pin(async {})
    }));

    let ws_sender_clone = ws_sender.clone();
    peer_connection.on_ice_candidate(Box::new(move |candidate| {
        let ws_sender_clone = ws_sender_clone.clone();
        Box::pin(async move {
            if let Some(candidate) = candidate {
                println!("New ICE candidate: {:?}", candidate);
                if let Ok(candidate_json) = candidate.to_json() {
                    let msg = json!({
                        "candidate": candidate_json,
                        "join_code": JOIN_CODE
                    })
                    .to_string();

                    if let Err(e) = ws_sender_clone
                        .lock()
                        .await
                        .send(Message::Text(msg.into()))
                        .await
                    {
                        println!("Error sending ICE candidate: {:?}", e);
                    }
                }
            }
        })
    }));

    let audio_track = Arc::new(TrackLocalStaticSample::new(
        RTCRtpCodecCapability {
            mime_type: "audio/opus".to_owned(),
            ..Default::default()
        },
        "audio".to_owned(),
        "webrtc-audio".to_owned(),
    ));

    let rtp_sender = peer_connection
        .add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal + Send + Sync>)
        .await?;

    let _rtcp_thread = tokio::spawn(async move {
        let mut rtcp_buf = vec![0u8; 1500];
        while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
        Result::<(), webrtc::Error>::Ok(())
    });

    let offer = peer_connection.create_offer(None).await?;
    println!("Created offer SDP:\n{}", offer.sdp);
    peer_connection.set_local_description(offer).await?;

    let mut gather_complete = peer_connection.gathering_complete_promise().await;
    let _ = gather_complete.recv().await;
    println!("ICE gathering complete");

    if let Some(local_desc) = peer_connection.local_description().await {
        println!("Sending offer SDP to remote peer");
        let offer_msg = json!({
            "offer": local_desc.sdp,
            "join_code": JOIN_CODE
        })
        .to_string();

        ws_sender
            .lock()
            .await
            .send(Message::Text(offer_msg.into()))
            .await?;
    } else {
        println!("Failed to get local description!");
        return Ok(());
    }

    let opus_encoder = Arc::new(tokio::sync::Mutex::new(
        Encoder::new(48000, Channels::Stereo, Application::Voip)
            .expect("Failed to create Opus encoder"),
    ));

    let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<f32>>(10);
    tokio::spawn(async move {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("No input device available");
        println!("Using input device: {}", device.name().unwrap());

        let config = configure_input_stream(&device);
        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let _ = audio_tx.blocking_send(data.to_vec());
                },
                move |err| {
                    eprintln!("Stream error: {}", err);
                },
                None,
            )
            .unwrap();

        stream.play().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(300));
    });

    let audio_track_clone = Arc::clone(&audio_track);
    let opus_encoder_clone = Arc::clone(&opus_encoder);

    tokio::spawn(async move {
        let mut buffer: Vec<f32> = Vec::new();

        while let Some(chunk) = audio_rx.recv().await {
            buffer.extend_from_slice(&chunk);

            // Only process when we have at least 1920 samples (20ms at 48kHz stereo)
            while buffer.len() >= 1920 {
                let frame: Vec<i16> = buffer.drain(..1920).map(|s| (s * 32767.0) as i16).collect();

                let mut opus_buffer = vec![0u8; 4000];
                let encoded_bytes = {
                    let mut encoder = opus_encoder_clone.lock().await;
                    match encoder.encode(&frame, &mut opus_buffer) {
                        Ok(n) => n,
                        Err(e) => {
                            eprintln!("Opus encoding failed: {:?}", e);
                            continue;
                        }
                    }
                };

                let sample = Sample {
                    data: Bytes::copy_from_slice(&opus_buffer[..encoded_bytes]),
                    timestamp: SystemTime::now(),
                    duration: Duration::from_millis(20),
                    packet_timestamp: 0,
                    prev_dropped_packets: 0,
                    prev_padding_packets: 0,
                };

                if let Err(e) = audio_track_clone.write_sample(&sample).await {
                    eprintln!("Error sending audio sample: {:?}", e);
                } else {
                    println!("✅ Sent {} bytes of Opus", encoded_bytes);
                }
            }
        }
    });

    println!("Listening for signaling messages");
    let mut shutdown = false;
    while !shutdown {
        if let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    println!("Received text message: {}", text);
                    let json_msg: Result<Value, _> = serde_json::from_str(&text);

                    if let Ok(json_msg) = json_msg {
                        if let Some(answer) = json_msg.get("answer").and_then(|v| v.as_str()) {
                            println!("Received SDP answer");
                            let answer = RTCSessionDescription::answer(answer.to_string()).unwrap();

                            match peer_connection.set_remote_description(answer).await {
                                Ok(_) => println!("Set remote description successfully"),
                                Err(e) => println!("Error setting remote description: {:?}", e),
                            }
                        }

                        if let Some(candidate) = json_msg.get("candidate") {
                            println!("Received ICE candidate: {:?}", candidate);

                            match serde_json::from_str::<RTCIceCandidateInit>(
                                &serde_json::to_string(candidate).unwrap(),
                            ) {
                                Ok(candidate_init) => {
                                    if let Err(e) =
                                        peer_connection.add_ice_candidate(candidate_init).await
                                    {
                                        println!("Error adding ICE candidate: {:?}", e);
                                    }
                                }
                                Err(e) => println!("Error parsing ICE candidate: {:?}", e),
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("WebSocket closed");
                    shutdown = true;
                }
                Err(e) => {
                    println!("WebSocket error: {:?}", e);
                    shutdown = true;
                }
                _ => {}
            }
        }
    }

    Ok(())
}
