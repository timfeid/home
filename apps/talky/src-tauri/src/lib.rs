// main.rs (or lib.rs)
use bytemuck::cast_slice;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use futures_util::{SinkExt, StreamExt};
use opus::{Application, Channels, Encoder};
use serde_json::{json, Value};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tungstenite::Bytes;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::media::Sample;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

use tauri::{Manager, State};

#[derive(Debug)]
enum AudioCommand {
    Start,
    Stop,
}

pub struct StreamingChannel {
    peer_connection: Arc<RTCPeerConnection>,
    audio_track: Arc<TrackLocalStaticSample>,
    ws_sender: Arc<
        Mutex<
            futures_util::stream::SplitSink<
                tokio_tungstenite::WebSocketStream<
                    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
                >,
                Message,
            >,
        >,
    >,
    audio_command_tx: mpsc::Sender<AudioCommand>,
    rt: Arc<tokio::runtime::Runtime>, // Dedicated runtime for all tasks.
    audio_stop_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
}

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

impl StreamingChannel {
    /// Creates a new StreamingChannel by spawning a dedicated runtime thread
    /// and running all our async tasks on that runtime.
    pub async fn new() -> Self {
        // Spawn a dedicated thread with its own multi-threaded runtime.
        let (tx, mut rx) = mpsc::channel::<StreamingChannel>(1);
        thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime");
            let rt = Arc::new(rt);
            let rt_clone = rt.clone();
            rt.clone().block_on(async move {
                // --- 1. Connect to the signaling server ---
                let ws_url = "ws://localhost:8080/soundhouse";
                println!("Connecting to signaling server at {}", ws_url);
                let (ws_stream, _) = connect_async(ws_url)
                    .await
                    .expect("Failed to connect to signaling server");
                let (ws_sender, mut ws_receiver) = ws_stream.split();
                let ws_sender = Arc::new(Mutex::new(ws_sender));
                let join_code = "room1";
                let init_msg = json!({
                    "join_code": join_code,
                    "role": "offerer"
                })
                .to_string();
                println!("Sending init message: {}", init_msg);
                ws_sender
                    .lock()
                    .await
                    .send(Message::Text(init_msg.into()))
                    .await
                    .expect("Failed to send init message");

                // Spawn a receiver loop on this runtime.

                // --- 2. Set up the WebRTC PeerConnection ---
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
                media_engine
                    .register_default_codecs()
                    .expect("Failed to register codecs");
                let registry = register_default_interceptors(Registry::new(), &mut media_engine)
                    .expect("Failed to register interceptors");
                let api = APIBuilder::new()
                    .with_media_engine(media_engine)
                    .with_interceptor_registry(registry)
                    .build();
                let peer_connection = Arc::new(
                    api.new_peer_connection(config)
                        .await
                        .expect("Failed to create peer connection"),
                );
                let pc = peer_connection.clone();
                let pc_receiver_handle = {
                let ws_sender_clone = ws_sender.clone();
                    rt_clone.spawn(async move {
                        let ws_sender_inner = ws_sender_clone.clone();
                        loop {
                            if let Some(msg) = ws_receiver.next().await {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        println!("Received text message: {}", text);
                                        let json_msg: Result<Value, _> =
                                            serde_json::from_str(&text);

                                        if let Ok(json_msg) = json_msg {
                                            if let Some(answer) =
                                                json_msg.get("answer").and_then(|v| v.as_str())
                                            {
                                                println!("Received SDP answer");
                                                let answer = RTCSessionDescription::answer(
                                                    answer.to_string(),
                                                )
                                                .unwrap();

                                                match pc.set_remote_description(answer).await {
                                                    Ok(_) => println!(
                                                        "Set remote description successfully"
                                                    ),
                                                    Err(e) => println!(
                                                        "Error setting remote description: {:?}",
                                                        e
                                                    ),
                                                }
                                            }

                                            if let Some(candidate) = json_msg.get("candidate") {
                                                println!("Received ICE candidate: {:?}", candidate);

                                                match serde_json::from_str::<RTCIceCandidateInit>(
                                                    &serde_json::to_string(candidate).unwrap(),
                                                ) {
                                                    Ok(candidate_init) => {
                                                        if let Err(e) = pc
                                                            .add_ice_candidate(candidate_init)
                                                            .await
                                                        {
                                                            println!(
                                                                "Error adding ICE candidate: {:?}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                    Err(e) => println!(
                                                        "Error parsing ICE candidate: {:?}",
                                                        e
                                                    ),
                                                }
                                            }

                                            if json_msg.get("type") == Some(&json!("active_clients")) {
                                                println!("[SignalingManager] Received active_clients message.");
                                                if let Some(clients) = json_msg.get("clients").and_then(|v| v.as_array()) {
                                                    if clients.iter().any(|c| c.get("role") == Some(&json!("answerer"))) {
                                                        println!("[SignalingManager] Answerer joined, resending offer...");
                                                        if let Some(local_desc) = pc.local_description().await {
                                                            let offer_msg = json!({
                                                                "offer": local_desc.sdp,
                                                                "join_code": "room1"
                                                            })
                                                            .to_string();
                                                            if let Err(e) = ws_sender_inner.lock().await.send(Message::Text(offer_msg.into())).await {
                                                                eprintln!("[SignalingManager] Failed to resend offer: {:?}", e);
                                                            }
                                                        } else {
                                                            eprintln!("[SignalingManager] No local description available to resend.");
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Ok(Message::Ping(data)) => {
                                        println!("[Signaling] Received Ping, sending Pong");
                                        if let Err(e) = ws_sender_inner
                                            .lock()
                                            .await
                                            .send(Message::Pong(data))
                                            .await
                                        {
                                            println!("[Signaling] Error sending Pong: {:?}", e);
                                        }
                                    }
                                    Ok(Message::Close(_)) => {
                                        println!("[Signaling] WebSocket closed");
                                        break;
                                    }
                                    Err(e) => {
                                        println!("[Signaling] WebSocket error: {:?}", e);
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                        println!("[Signaling] Receiver loop ended");
                    })
                };
                println!("Created peer connection");

                // Log state changes.
                {
                    let pc = Arc::clone(&peer_connection);
                    peer_connection.on_peer_connection_state_change(Box::new(move |state| {
                        println!("Peer Connection State Changed: {:?}", state);
                        if state == RTCPeerConnectionState::Failed {
                            println!("Peer connection failed, attempting restart...");
                        }
                        Box::pin(async {})
                    }));
                }

                // ICE candidate handling.
                let ws_sender_clone = ws_sender.clone();
                {
                    let join_code = join_code.to_string();
                    peer_connection.on_ice_candidate(Box::new(move |candidate| {
                        let ws_sender_inner = ws_sender_clone.clone();
                        let join_code = join_code.clone();
                        Box::pin(async move {
                            if let Some(candidate) = candidate {
                                println!("New ICE candidate: {:?}", candidate);
                                if let Ok(candidate_json) = candidate.to_json() {
                                    let msg = json!({
                                        "candidate": candidate_json,
                                        "join_code": join_code
                                    })
                                    .to_string();
                                    if let Err(e) = ws_sender_inner
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
                }

                // --- 3. Create the audio track and add to PeerConnection ---
                let audio_track = Arc::new(TrackLocalStaticSample::new(
                    RTCRtpCodecCapability {
                        mime_type: "audio/opus".to_owned(),
                        ..Default::default()
                    },
                    "audio".to_owned(),
                    "webrtc-audio".to_owned(),
                ));
                println!("Created audio track");
                let rtp_sender = peer_connection
                    .add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal + Send + Sync>)
                    .await
                    .expect("Failed to add audio track");
                rt_clone.spawn(async move {
                    let mut rtcp_buf = vec![0u8; 1500];
                    while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
                });

                // --- 4. Create and send an SDP offer ---
                let offer = peer_connection
                    .create_offer(None)
                    .await
                    .expect("Failed to create offer");
                println!("Created offer SDP:\n{}", offer.sdp);
                peer_connection
                    .set_local_description(offer)
                    .await
                    .expect("Failed to set local description");
                let mut gather_complete = peer_connection.gathering_complete_promise().await;
                let _ = gather_complete.recv().await;
                println!("ICE gathering complete");
                if let Some(local_desc) = peer_connection.local_description().await {
                    println!("Sending offer SDP to remote peer");
                    let offer_msg = json!({
                        "offer": local_desc.sdp,
                        "join_code": join_code
                    })
                    .to_string();
                    ws_sender
                        .lock()
                        .await
                        .send(Message::Text(offer_msg.into()))
                        .await
                        .expect("Failed to send offer message");
                } else {
                    println!("Failed to get local description");
                }

                // --- 5. Set up audio capture ---
                let (audio_command_tx, mut audio_command_rx) = mpsc::channel::<AudioCommand>(10);
                let audio_track_clone = Arc::clone(&audio_track);
                let opus_encoder = Arc::new(tokio::sync::Mutex::new(
                    Encoder::new(48000, Channels::Stereo, Application::Voip)
                        .expect("Failed to create Opus encoder"),
                ));
                let (audio_command_tx, mut audio_command_rx) = mpsc::channel::<AudioCommand>(10);
                let opus_encoder = Arc::new(Mutex::new(Encoder::new(48000, Channels::Stereo, Application::Voip).expect("Failed to create Opus encoder")));
                let audio_stop_tx: Arc<Mutex<Option<mpsc::Sender<()>>>> = Arc::new(Mutex::new(None));
                let audio_track_clone = Arc::clone(&audio_track);
                let audio_stop_tx_clone = audio_stop_tx.clone();
                let opus_encoder_clone = opus_encoder.clone();

                rt_clone.spawn(async move {
                    while let Some(command) = audio_command_rx.recv().await {
                        match command {
                            AudioCommand::Start => {
                                println!("Starting audio capture...");
                                let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<f32>>(10);
                                let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

                                {
                                    let mut guard = audio_stop_tx_clone.lock().await;
                                    *guard = Some(stop_tx);
                                }

                                std::thread::spawn(move || {
                                    let host = cpal::default_host();
                                    let device = host.default_input_device().expect("No input device");
                                    let config = configure_input_stream(&device);

                                    let stream = device.build_input_stream(
                                        &config.into(),
                                        move |data: &[f32], _| {
                                            let _ = audio_tx.blocking_send(data.to_vec());
                                        },
                                        move |err| {
                                            eprintln!("Stream error: {}", err);
                                        },
                                        None,
                                    ).expect("Failed to build input stream");

                                    stream.play().expect("Failed to start stream");

                                });

                                let opus_encoder_clone=opus_encoder_clone.clone();
                                let audio_track_clone=audio_track_clone.clone();
                                tokio::spawn(async move {
let opus_encoder_clone = opus_encoder_clone.clone();
let audio_track_clone = audio_track_clone.clone();
                    let mut buffer: Vec<f32> = Vec::new();
                                    loop {
                                        tokio::select! {
                                            _ = stop_rx.recv() => {
                                                println!("Received stop signal, ending stream loop.");
                                                break;
                                            }
                                            Some(chunk) = audio_rx.recv() => {
                                                buffer.extend_from_slice(&chunk);
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
                                                    }
                                                }
                                            }
                                            else => break,
                                        }
                                    }
                                });
                            }
                            AudioCommand::Stop => {
                                println!("Stopping audio capture...");
                                if let Some(stop_tx) = audio_stop_tx_clone.lock().await.take() {
                                    let _ = stop_tx.send(()).await;
                                }
                                break;
                            }
                        }
                    }
                });

                // Construct the StreamingChannel and send it over the channel.
                let channel = StreamingChannel {
                    peer_connection,
                    audio_track,
                    ws_sender,
                    audio_command_tx,
                    rt: rt_clone, // store our dedicated runtime
                    audio_stop_tx,
                };
                // Send the constructed channel back.
                tx.send(channel)
                    .await
                    .expect("Failed to send StreamingChannel");
            });
        });
        // Wait and receive the StreamingChannel from our thread.
        let channel = rx.recv().await.expect("Failed to receive StreamingChannel");
        channel
    }

    pub async fn start(&self) {
        println!("StreamingChannel: start()");
        let _ = self.audio_command_tx.send(AudioCommand::Start).await;
    }

    pub async fn stop(&self) {
        println!("StreamingChannel: stop()");
        let _ = self.audio_command_tx.send(AudioCommand::Stop).await;
    }
}

// --- Tauri Commands and run() ---
#[tauri::command]
async fn start_record(state: State<'_, StreamingChannel>) -> Result<(), String> {
    println!("Tauri command: start_record");
    state.start().await;
    Ok(())
}

#[tauri::command]
async fn end_record(state: State<'_, StreamingChannel>) -> Result<(), String> {
    println!("Tauri command: end_record");
    state.stop().await;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(async {
                    println!("🚀 Initializing StreamingChannel in background...");
                    let manager = StreamingChannel::new().await;
                    println!("✅ StreamingChannel ready!");
                    app_handle.manage(manager);
                });
            });
            println!("✅ Tauri app setup complete.");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![start_record, end_record])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
