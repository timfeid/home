use anyhow::{Context, Result};
use bytemuck::cast_slice;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig, SupportedStreamConfig};
use futures_util::{SinkExt, StreamExt};
use opus::{Application, Channels, Encoder};
use serde::{Deserialize, Serialize};

use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::{StoreBuilder, StoreExt};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCaptureConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: u32,
    pub capture_mode: CaptureMode,
    pub voice_activity_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CaptureMode {
    PushToTalk,
    VoiceActivated,
    Continuous,
}

pub struct AudioProcessor {
    config: AudioCaptureConfig,
    opus_encoder: Arc<Mutex<Encoder>>,
}

impl AudioProcessor {
    pub fn new(config: AudioCaptureConfig) -> Result<Self> {
        let opus_encoder = Encoder::new(
            config.sample_rate,
            match config.channels {
                1 => Channels::Mono,
                2 => Channels::Stereo,
                _ => return Err(anyhow::anyhow!("Unsupported number of channels")),
            },
            Application::Voip,
        )?;

        Ok(Self {
            config,
            opus_encoder: Arc::new(Mutex::new(opus_encoder)),
        })
    }

    fn calculate_rms(samples: &[f32]) -> f32 {
        let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    fn rms_to_decibels(rms: f32) -> f32 {
        20.0 * rms.log10()
    }

    pub fn should_send_audio(&self, samples: &[f32], is_push_to_talk_active: bool) -> bool {
        match self.config.capture_mode {
            CaptureMode::PushToTalk => is_push_to_talk_active,
            CaptureMode::VoiceActivated => {
                let rms = Self::calculate_rms(samples);
                let db_level = Self::rms_to_decibels(rms);
                db_level > self.config.voice_activity_threshold
            }
            CaptureMode::Continuous => true,
        }
    }

    pub async fn encode_samples(&self, samples: &[f32]) -> Result<Vec<u8>> {
        let frame: Vec<i16> = samples.iter().map(|&s| (s * 32767.0) as i16).collect();

        let mut opus_buffer = vec![0u8; 4000];

        let encoded_bytes = {
            let mut encoder = self.opus_encoder.lock().await;
            encoder
                .encode(&frame, &mut opus_buffer)
                .context("Opus encoding failed")?
        };

        Ok(opus_buffer[..encoded_bytes].to_vec())
    }
}

pub struct WebRTCManager {
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
    audio_processor: Arc<AudioProcessor>,
    capture_control: Arc<Mutex<CaptureControl>>,
    pub cancellation_token: tokio_util::sync::CancellationToken,
}

pub struct CaptureControl {
    push_to_talk_active: bool,
}

impl CaptureControl {
    pub fn new() -> Self {
        Self {
            push_to_talk_active: false,
        }
    }

    pub fn set_push_to_talk(&mut self, active: bool) {
        self.push_to_talk_active = active;
    }

    pub fn is_push_to_talk_active(&self) -> bool {
        self.push_to_talk_active
    }
}

impl WebRTCManager {
    pub async fn start_audio_capture(&self) -> Result<Arc<dyn cpal::traits::StreamTrait>> {
        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .context("No input device available")?;

        let supported_config = input_device.default_input_config()?;

        let use_stereo = supported_config.channels() == 2;
        let channels = if use_stereo { 2 } else { 1 };

        let stream_config = StreamConfig {
            channels,
            sample_rate: SampleRate(48000),
            buffer_size: BufferSize::Fixed(1024),
        };

        let (tx, mut rx) = mpsc::channel::<Vec<f32>>(100);
        let audio_processor = Arc::clone(&self.audio_processor);
        let audio_track = Arc::clone(&self.audio_track);
        let capture_control = Arc::clone(&self.capture_control);
        let cancellation_token = self.cancellation_token.clone();

        tokio::spawn(async move {
            println!("Audio processing task started");
            let mut buffer: Vec<f32> = Vec::new();

            let frame_size = match channels {
                1 => 960,
                2 => 1920,
                _ => {
                    eprintln!("Unsupported channel count");
                    return;
                }
            };

            while let Some(chunk) = rx.recv().await {
                if cancellation_token.is_cancelled() {
                    println!("Cancellation token triggered");
                    break;
                }

                buffer.extend_from_slice(&chunk);

                while buffer.len() >= frame_size {
                    let frame_f32: Vec<f32> = buffer.drain(..frame_size).collect();

                    let capture_control_guard = capture_control.lock().await;
                    let should_send = audio_processor.should_send_audio(
                        &frame_f32,
                        capture_control_guard.is_push_to_talk_active(),
                    );

                    if !should_send {
                        continue;
                    }

                    let frame_i16: Vec<i16> = frame_f32
                        .iter()
                        .map(|&s| (s * 32767.0).clamp(i16::MIN as f32, i16::MAX as f32) as i16)
                        .collect();

                    let mut opus_buffer = vec![0u8; 4000];
                    let encoded_bytes = {
                        let mut encoder = audio_processor.opus_encoder.lock().await;
                        match encoder.encode(&frame_i16, &mut opus_buffer) {
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

                    if let Err(e) = audio_track.write_sample(&sample).await {
                        eprintln!("Error writing audio sample: {:?}", e);
                    }
                }
            }
        });

        let sender = tx.clone();
        let stream = input_device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Err(e) = sender.try_send(data.to_vec()) {
                    eprintln!("Audio channel is full, dropping samples: {:?}", e);
                }
            },
            |err| eprintln!("Audio input stream error: {:?}", err),
            None,
        )?;

        stream.play()?;
        println!("Started audio capture");

        Ok(Arc::new(stream))
    }

    pub async fn stop_audio_capture(&self) {
        self.cancellation_token.cancel();
    }

    pub async fn process_audio_samples(&self, samples: &[f32]) -> Result<()> {
        let capture_control = self.capture_control.lock().await;
        let should_send = self
            .audio_processor
            .should_send_audio(samples, capture_control.is_push_to_talk_active());

        if !should_send {
            return Ok(());
        }

        let encoded_samples = self.audio_processor.encode_samples(samples).await?;

        let sample = Sample {
            packet_timestamp: 0,
            prev_padding_packets: 0,
            prev_dropped_packets: 0,
            data: Bytes::from(encoded_samples),
            duration: Duration::from_millis(20),
            timestamp: SystemTime::now(),
        };

        self.audio_track.write_sample(&sample).await?;

        Ok(())
    }

    pub async fn set_capture_mode(&self, mode: CaptureMode) -> Result<()> {
        Ok(())
    }

    pub async fn new(
        config: AudioCaptureConfig,
        signaling_url: &str,
        join_code: String,
    ) -> Result<Self> {
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let cancellation_token_clone = cancellation_token.clone();

        let rtc_config = RTCConfiguration {
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
            .context("Failed to register codecs")?;

        let registry = register_default_interceptors(Registry::new(), &mut media_engine)
            .context("Failed to register interceptors")?;

        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        let peer_connection = Arc::new(
            api.new_peer_connection(rtc_config)
                .await
                .context("Failed to create peer connection")?,
        );

        let (ws_stream, _) =
            tokio::time::timeout(Duration::from_secs(10), connect_async(signaling_url))
                .await
                .context("WebSocket connection timeout")?
                .context("Failed to connect to signaling server")?;

        let (ws_sender, mut ws_receiver) = ws_stream.split();
        let ws_sender = Arc::new(Mutex::new(ws_sender));

        let init_msg = json!({
            "type": "init",
            "auth_code": join_code.clone(),
        })
        .to_string();
        ws_sender
            .lock()
            .await
            .send(Message::Text(init_msg.into()))
            .await
            .context("Failed to send init message")?;

        let ws_sender_clone = ws_sender.clone();
        {
            let join_code_inner = join_code.clone();
            peer_connection.on_ice_candidate(Box::new(move |candidate| {
                let ws_sender_inner = ws_sender_clone.clone();
                let join_code = join_code_inner.clone();
                Box::pin(async move {
                    let join_code = join_code.clone();
                    if let Some(candidate) = candidate {
                        println!("New ICE candidate: {:?}", candidate);
                        if let Ok(candidate_json) = candidate.to_json() {
                            let msg = json!({
                                "candidate": candidate_json,
                                "join_code": &join_code
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
            .await
            .context("Failed to add audio track")?;

        let _rtcp_thread = tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
            Result::<(), webrtc::Error>::Ok(())
        });

        let audio_processor = Arc::new(AudioProcessor::new(config)?);
        let capture_control = Arc::new(Mutex::new(CaptureControl::new()));

        let pc_clone = Arc::clone(&peer_connection);
        let ws_sender_clone = ws_sender.clone();

        let join_code_inner = join_code.clone();
        tokio::spawn(async move {
            let join_code = join_code_inner.clone();
            println!("DEBUG: Starting WebSocket receiver loop");

            loop {
                println!("DEBUG: Top of loop");
                tokio::select! {
                    _ = cancellation_token_clone.cancelled() => {
                        println!("CRITICAL: WebSocket receiver loop cancelled");
                        break;
                    }
                    Some(msg_result) = ws_receiver.next() => {
                        println!("DEBUG: Received a message in stream");
                        match msg_result {
                            Ok(msg) => {
                                println!("DEBUG: Message is Ok");
                                match msg {
                                    Message::Text(text) => {
                                        println!("VERBOSE: Received WebSocket text message: {}", text);

                                        match serde_json::from_str::<Value>(&text) {
                                            Ok(json_msg) => {
                                                println!("VERBOSE: Parsed JSON message: {:#?}", json_msg);

                                                match json_msg.get("type").and_then(|t| t.as_str()) {
                                                    Some("active_clients") => {
                                                        println!("INFO: Received active_clients message.");
                                                        if let Some(clients) = json_msg.get("clients").and_then(|v| v.as_array()) {
                                                            println!("DEBUG: Clients array: {:?}", clients);
                                                            if clients.iter().any(|c| c.get("role") == Some(&json!("answerer"))) {
                                                                println!("INFO: Answerer detected, attempting to send offer...");
                                                                if let Some(local_desc) = pc_clone.local_description().await {
                                                                    let offer_msg = json!({
                                                                        "type": "offer",
                                                                        "offer": local_desc.sdp,
                                                                        "join_code": join_code.clone()
                                                                    })
                                                                    .to_string();

                                                                    println!("DEBUG: Offer message: {}", offer_msg);
                                                                    match ws_sender_clone.lock().await.send(Message::Text(offer_msg.into())).await {
                                                                        Ok(_) => println!("INFO: Offer sent successfully"),
                                                                        Err(e) => eprintln!("ERROR: Failed to send offer: {:?}", e),
                                                                    }
                                                                } else {
                                                                    println!("WARNING: No local description available");
                                                                }
                                                            } else {
                                                                println!("DEBUG: No answerer found in clients");
                                                            }
                                                        } else {
                                                            println!("WARNING: No clients array in message");
                                                        }
                                                    }
                                                    Some("answer") => {
                                                        if let Some(answer) = json_msg.get("answer").and_then(|v| v.as_str()) {

                                                            println!("Received SDP answer");
                                                            let answer = RTCSessionDescription::answer(answer.to_string()).unwrap();

                                                            match pc_clone.set_remote_description(answer).await {
                                                                Ok(_) => println!("Set remote description successfully"),
                                                                Err(e) => println!("Error setting remote description: {:?}", e),
                                                            }
                                                        }
                                                    }
                                                    Some("candidate") => {
                                                        println!("INFO: Received ICE candidate");
                                                        if let Some(candidate_obj) = json_msg.get("candidate") {
                                                            match serde_json::from_str::<RTCIceCandidateInit>(
                                                                &serde_json::to_string(candidate_obj).unwrap()
                                                            ) {
                                                                Ok(candidate_init) => {
                                                                    if let Err(e) = pc_clone.add_ice_candidate(candidate_init).await {
                                                                        eprintln!("ERROR: Adding ICE candidate failed: {:?}", e);
                                                                    }
                                                                }
                                                                Err(e) => eprintln!("ERROR: Parsing ICE candidate failed: {:?}", e),
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        println!("WARNING: Unhandled message type: {}",
                                                            json_msg.get("type").and_then(|t| t.as_str()).unwrap_or("unknown"));
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                eprintln!("ERROR: Failed to parse JSON message: {:?}", e);
                                                eprintln!("RAW message content: {}", text);
                                            }
                                        }
                                    },
                                    Message::Ping(ping_data) => {
                                        println!("INFO: Received Ping, sending Pong");
                                        if let Err(e) = ws_sender_clone.lock().await.send(Message::Pong(ping_data)).await {
                                            eprintln!("ERROR: Sending Pong failed: {:?}", e);
                                            break;
                                        }
                                    },
                                    Message::Close(_) => {
                                        println!("INFO: WebSocket connection closed");
                                        break;
                                    },
                                    _ => {
                                        println!("WARNING: Received unhandled message type");
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("ERROR: WebSocket stream error: {:?}", e);
                                break;
                            }
                        }
                    }
                    else => {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        println!("TRACE: Idle loop iteration");
                    }
                }
            }

            println!("CRITICAL: WebSocket receiver loop terminated");
        });

        let offer = peer_connection
            .create_offer(None)
            .await
            .context("Failed to create offer")?;

        peer_connection
            .set_local_description(offer)
            .await
            .context("Failed to set local description")?;

        let mut gather_complete = peer_connection.gathering_complete_promise().await;
        let _ = gather_complete.recv().await;

        if let Some(local_desc) = peer_connection.local_description().await {
            let offer_msg = json!({
                "offer": local_desc.sdp,
                "join_code": &join_code
            })
            .to_string();

            ws_sender
                .lock()
                .await
                .send(Message::Text(offer_msg.into()))
                .await
                .context("Failed to send offer message")?;
        }

        Ok(Self {
            peer_connection,
            audio_track,
            ws_sender,
            audio_processor,
            capture_control,
            cancellation_token,
        })
    }

    pub async fn stop(&self) {}

    pub async fn shutdown(&self) {
        self.cancellation_token.cancel();

        println!("WebRTCManager shutting down");
    }
}

#[tauri::command]
async fn set_refresh_token(app: AppHandle, token: String) -> Result<(), tauri_plugin_store::Error> {
    // Define the path for the store (adjust as needed)
    let path = app
        .path()
        .app_data_dir()
        .expect("unable to find data dir")
        .join("data.json");

    let store = app.store(path)?;
    store.set("refresh_token".to_string(), json!(token));
    Ok(())
}

#[tauri::command]
async fn get_refresh_token(app: AppHandle) -> Result<Option<Value>, tauri_plugin_store::Error> {
    let path = app
        .path()
        .app_data_dir()
        .expect("unable to find data dir")
        .join("data.json");

    app.store(path).and_then(|s| Ok(s.get("refresh_token")))
}

#[tauri::command]
async fn configure_audio_capture(
    config: AudioCaptureConfig,
    state: tauri::State<'_, WebRTCManager>,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
async fn toggle_push_to_talk(
    active: bool,
    state: tauri::State<'_, WebRTCManager>,
) -> Result<(), String> {
    let mut capture_control = state.capture_control.lock().await;

    capture_control.set_push_to_talk(active);

    println!("Push-to-talk {}activated", if active { "" } else { "de" });

    Ok(())
}

pub fn run(manager: WebRTCManager) {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            app.handle().manage(manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            configure_audio_capture,
            toggle_push_to_talk,
            get_refresh_token,
            set_refresh_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
