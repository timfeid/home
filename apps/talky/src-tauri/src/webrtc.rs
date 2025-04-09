// modules/webrtc.rs
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, Mutex, RwLock};
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
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

use crate::audio::{AudioProcessor, CaptureControl, AUDIO_STREAM};
use crate::config::{AudioCaptureConfig, CaptureMode};
use crate::signaling::{self, WebSocketSender};

pub struct WebRTCAudio {
    pub peer_connection: Arc<RTCPeerConnection>,
    pub audio_track: Arc<TrackLocalStaticSample>,
    pub audio_processor: Arc<AudioProcessor>,
    pub capture_control: Arc<Mutex<CaptureControl>>,
    pub ws_sender: Arc<Mutex<WebSocketSender>>,
}

pub struct WebRTCManager {
    pub signaling_url: String,
    pub audio: RwLock<Option<WebRTCAudio>>,
    pub cancellation_token: tokio_util::sync::CancellationToken,
    pub audio_capture_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl WebRTCManager {
    pub async fn new(signaling_url: &str) -> Result<Self> {
        let cancellation_token = tokio_util::sync::CancellationToken::new();
        let audio_capture_task = Arc::new(Mutex::new(None));

        Ok(Self {
            signaling_url: signaling_url.to_string(),
            audio: RwLock::new(None),
            cancellation_token,
            audio_capture_task,
        })
    }

    pub async fn start(
        &self,
        config: AudioCaptureConfig,
        auth_code: String,
        channel_id: String,
        niche_id: String,
    ) -> Result<()> {
        // Create WebRTC configuration
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

        // Setup media engine
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

        // Create peer connection
        let peer_connection = Arc::new(
            api.new_peer_connection(rtc_config)
                .await
                .context("Failed to create peer connection")?,
        );

        // Connect to signaling server
        let (ws_stream, _) =
            tokio::time::timeout(Duration::from_secs(10), connect_async(&self.signaling_url))
                .await
                .context("WebSocket connection timeout")?
                .context("Failed to connect to signaling server")?;

        let (ws_sender, mut ws_receiver) = ws_stream.split();
        let ws_sender = Arc::new(Mutex::new(ws_sender));

        // Send initial messages
        let init_msg = json!({
            "type": "init",
            "auth_code": auth_code.clone(),
        })
        .to_string();
        ws_sender
            .lock()
            .await
            .send(Message::Text(init_msg.into()))
            .await
            .context("Failed to send init message")?;

        let msg = json!({
            "type": "join",
            "channel_id": &channel_id,
            "role": "offerer"
        })
        .to_string();

        ws_sender
            .lock()
            .await
            .send(Message::Text(msg.into()))
            .await
            .context("Failed to send join message")?;

        // Setup ICE candidate handling
        let ws_sender_clone = ws_sender.clone();
        {
            let channel_id_inner = channel_id.clone();
            let niche_id = niche_id.clone();
            peer_connection.on_ice_candidate(Box::new(move |candidate| {
                let ws_sender_inner = ws_sender_clone.clone();
                let channel_id = channel_id_inner.clone();
                let niche_id = niche_id.clone();
                Box::pin(async move {
                    let channel_id = channel_id.clone();
                    if let Some(candidate) = candidate {
                        println!("New ICE candidate: {:?}", candidate);
                        if let Ok(candidate_json) = candidate.to_json() {
                            let msg = json!({
                                "type": "candidate",
                                "candidate": candidate_json,
                                "channel_id": &channel_id,
                                "niche_id": &niche_id
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

        // Create audio track
        let audio_track = Arc::new(TrackLocalStaticSample::new(
            RTCRtpCodecCapability {
                mime_type: "audio/opus".to_owned(),
                ..Default::default()
            },
            "audio".to_owned(),
            "webrtc-audio".to_owned(),
        ));

        // Add track to peer connection
        let rtp_sender = peer_connection
            .add_track(Arc::clone(&audio_track) as Arc<dyn TrackLocal + Send + Sync>)
            .await
            .context("Failed to add audio track")?;

        // RTCP handling
        let _rtcp_thread = tokio::spawn(async move {
            let mut rtcp_buf = vec![0u8; 1500];
            while let Ok((_, _)) = rtp_sender.read(&mut rtcp_buf).await {}
            Result::<(), webrtc::Error>::Ok(())
        });

        // Create audio processor
        let audio_processor = Arc::new(AudioProcessor::new(config)?);
        let capture_control = Arc::new(Mutex::new(CaptureControl::new()));

        // Set up WebSocket message handling
        let pc_clone = Arc::clone(&peer_connection);
        let ws_sender_clone = ws_sender.clone();
        let cancellation_token_clone = self.cancellation_token.clone();

        let channel_id_inner = channel_id.clone();
        let niche_id_inner = niche_id.clone();
        tokio::spawn(async move {
            let channel_id = channel_id_inner.clone();
            let niche_id = niche_id_inner.clone();
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
                                                    Some("active_channels") => {
                                                        println!("INFO: Received active_channels message.");
                                                        if let Some(channels) = json_msg.get("channels").and_then(|v| v.as_object()) {
                                                            println!("DEBUG: Channels object: {:?}", channels);
                                                            if let Some(channel) = channels.get(&channel_id) {
                                                                if let Some(users) = channel.get("users").and_then(|v| v.as_array()) {
                                                                    println!("DEBUG: Users array: {:?}", users);
                                                                    if users.iter().any(|u| u.get("role") == Some(&json!("answerer"))) {
                                                                        println!("INFO: Answerer detected, attempting to send offer...");
                                                                        if let Some(local_desc) = pc_clone.local_description().await {
                                                                            let offer_msg = json!({
                                                                                "type": "offer",
                                                                                "offer": local_desc.sdp,

                                                                                "niche_id": &niche_id,
                                                                                "channel_id": &channel_id,

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
                                                                        println!("DEBUG: No answerer found in users");
                                                                    }
                                                                } else {
                                                                    println!("WARNING: No users array in channel");
                                                                }
                                                            } else {
                                                                println!("WARNING: Channel ID not found in channels");
                                                            }
                                                        } else {
                                                            println!("WARNING: No channels object in message");
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

        // Create and set local description (offer)
        let offer = peer_connection
            .create_offer(None)
            .await
            .context("Failed to create offer")?;

        peer_connection
            .set_local_description(offer)
            .await
            .context("Failed to set local description")?;

        // Wait for ICE gathering to complete
        let mut gather_complete = peer_connection.gathering_complete_promise().await;
        let _ = gather_complete.recv().await;

        // Send offer
        if let Some(local_desc) = peer_connection.local_description().await {
            let offer_msg = json!({
                "type": "offer",
                "niche_id": &niche_id,
                "offer": &local_desc.sdp,
                "channel_id": channel_id.clone()
            })
            .to_string();

            ws_sender
                .lock()
                .await
                .send(Message::Text(offer_msg.into()))
                .await
                .context("Failed to send offer message")?;
        }

        // Store WebRTCAudio instance
        *self.audio.write().await = Some(WebRTCAudio {
            peer_connection,
            audio_track,
            audio_processor,
            capture_control,
            ws_sender,
        });

        Ok(())
    }

    pub async fn start_audio_capture(self: Arc<Self>) -> Result<()> {
        println!("[AudioCapture] Initializing input stream...");

        let host = cpal::default_host();
        let input_device = host
            .default_input_device()
            .context("No input device available")?;
        println!(
            "[AudioCapture] Using input device: {}",
            input_device.name()?
        );

        let supported_config = input_device.default_input_config()?;
        println!("[AudioCapture] Supported config: {:?}", supported_config);

        let stream_config = StreamConfig {
            channels: supported_config.channels(),
            sample_rate: supported_config.sample_rate(),
            buffer_size: BufferSize::Fixed(1024),
        };

        let (tx, mut rx) = mpsc::channel::<Vec<f32>>(100);

        if let Some(audio) = &*self.audio.read().await {
            let audio_processor = audio.audio_processor.clone();
            let audio_track = audio.audio_track.clone();
            let capture_control = audio.capture_control.clone();
            let cancellation_token = self.cancellation_token.clone();
            let audio_capture_task = self.audio_capture_task.clone();

            let manager_clone = self.clone();
            let task_handle = tokio::spawn(async move {
                let mut buffer = Vec::new();
                let frame_size = match supported_config.channels() {
                    1 => 960,
                    2 => 1920,
                    _ => return,
                };
                while let Some(mut chunk) = rx.recv().await {
                    buffer.append(&mut chunk);
                    while buffer.len() >= frame_size {
                        let frame = buffer.drain(0..frame_size).collect::<Vec<f32>>();
                        let is_ptt_active = capture_control.lock().await.is_push_to_talk_active();

                        if audio_processor.should_send_audio(&frame, is_ptt_active) {
                            if let Ok(encoded) = audio_processor.encode_samples(&frame).await {
                                let sample = Sample {
                                    data: Bytes::from(encoded),
                                    duration: Duration::from_millis(20),
                                    timestamp: SystemTime::now(),
                                    ..Default::default()
                                };
                                let _ = audio_track.write_sample(&sample).await;
                            }
                        }
                    }
                }

                println!("[AudioStream] Receiver loop exited");
            });

            *audio_capture_task.lock().await = Some(task_handle);

            let stream = input_device.build_input_stream(
                &stream_config,
                move |data: &[f32], _| {
                    println!("[AudioStream] Callback with {} samples", data.len());
                    if let Err(e) = tx.try_send(data.to_vec()) {
                        eprintln!("[AudioStream] Failed to send audio data: {:?}", e);
                    }
                },
                move |err| {
                    eprintln!("[AudioStream] Stream error: {:?}", err);
                },
                None,
            )?;

            stream.play()?;

            AUDIO_STREAM.with(|cell| {
                *cell.borrow_mut() = Some(stream);
            });

            println!("[AudioCapture] Stream started");
            Ok(())
        } else {
            Err(anyhow::anyhow!("No audio session initialized"))
        }
    }

    pub async fn stop_audio_capture(&self) {
        self.cancellation_token.cancel();
        if let Some(handle) = self.audio_capture_task.lock().await.take() {
            handle.await.ok();
        }

        AUDIO_STREAM.with(|cell| {
            *cell.borrow_mut() = None;
        });
    }

    pub async fn process_audio_samples(&self, samples: &[f32]) -> Result<()> {
        println!("samples received");
        if let Some(audio) = &*self.audio.read().await {
            let capture_control = audio.capture_control.lock().await;
            let should_send = audio
                .audio_processor
                .should_send_audio(samples, capture_control.is_push_to_talk_active());

            if !should_send {
                return Ok(());
            }

            let encoded_samples = audio.audio_processor.encode_samples(samples).await?;

            let sample = Sample {
                packet_timestamp: 0,
                prev_padding_packets: 0,
                prev_dropped_packets: 0,
                data: Bytes::from(encoded_samples),
                duration: Duration::from_millis(20),
                timestamp: SystemTime::now(),
            };

            audio.audio_track.write_sample(&sample).await?;

            return Ok(());
        }

        Err(anyhow::anyhow!("not streaming yet"))
    }

    pub async fn set_capture_mode(&self, _mode: CaptureMode) -> Result<()> {
        Ok(())
    }

    pub async fn shutdown(&self) {
        self.cancellation_token.cancel();

        AUDIO_STREAM.with(|cell| {
            *cell.borrow_mut() = None;
        });

        if let Some(handle) = self.audio_capture_task.lock().await.take() {
            handle.await.ok();
        }

        println!("WebRTCManager shutting down");
    }
}
