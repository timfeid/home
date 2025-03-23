use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde_json::{json, Value};
use std::{
    sync::{mpsc::channel, Arc},
    thread,
};
use tokio::{
    net::TcpStream,
    sync::Mutex,
    time::{interval, sleep, Duration},
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use webrtc::{
    ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit},
    peer_connection::{sdp::session_description::RTCSessionDescription, RTCPeerConnection},
};

pub struct SignalingManager {
    ws_sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    rt: Arc<tokio::runtime::Runtime>,
}

impl SignalingManager {
    pub fn new(
        peer_connection: Arc<RTCPeerConnection>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Use a standard mpsc channel to pass the constructed SignalingManager back.
        let (tx, rx) = channel();

        thread::spawn(move || {
            // Create a multi-threaded runtime. This is safe to create here because we are in a new thread.
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime");
            let rt = Arc::new(rt);
            let rt_clone = rt.clone();

            // Block on the async work.
            let manager = rt.block_on(async {
                println!("[SignalingManager] Connecting to ws://localhost:8080/soundhouse...");
                let (ws_stream, _) = connect_async("ws://localhost:8080/soundhouse")
                    .await
                    .expect("Failed to connect");
                let (ws_sender, ws_receiver) = ws_stream.split();
                let ws_sender = Arc::new(Mutex::new(ws_sender));

                let init_msg = json!({
                    "join_code": "room1",
                    "role": "offerer"
                })
                .to_string();
                println!("[SignalingManager] Sending init message: {}", init_msg);
                {
                    let mut sender = ws_sender.lock().await;
                    sender
                        .send(Message::Text(init_msg.into()))
                        .await
                        .expect("Failed to send init message");
                }

                // Spawn the receiver loop (and a heartbeat task) on this runtime.
                Self::spawn_receiver(
                    ws_receiver,
                    ws_sender.clone(),
                    peer_connection,
                    rt_clone.clone(),
                );

                SignalingManager {
                    ws_sender,
                    rt: rt_clone,
                }
            });
            tx.send(manager)
                .expect("Failed to send manager over channel");
        });

        // Wait for the thread to send the manager.
        let manager = rx.recv()?;
        Ok(manager)
    }

    /// Called when a new ICE candidate is generated.
    pub async fn on_ice_candidate(&self, candidate: Option<RTCIceCandidate>) {
        if let Some(candidate) = candidate {
            println!("New ICE candidate: {:?}", candidate);
            if let Ok(candidate_json) = candidate.to_json() {
                let msg = json!({
                    "candidate": candidate_json,
                    "join_code": "room1"
                })
                .to_string();

                if let Err(e) = self
                    .ws_sender
                    .lock()
                    .await
                    .send(Message::Text(msg.into()))
                    .await
                {
                    println!("Error sending ICE candidate: {:?}", e);
                }
            }
        }
    }

    /// Send an arbitrary message over the signaling channel.
    pub async fn send(&self, msg: String) {
        if let Err(e) = self
            .ws_sender
            .lock()
            .await
            .send(Message::Text(msg.clone().into()))
            .await
        {
            println!("Error sending message: {:?}", e);
        } else {
            println!("Sent to signal: {}", msg);
        }
    }

    fn spawn_receiver(
        mut ws_receiver: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        ws_sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
        peer_connection: Arc<RTCPeerConnection>,
        rt: Arc<tokio::runtime::Runtime>,
    ) {
        rt.spawn(async move {
            println!("[SignalingManager] Receiver loop started.");
            while let Some(result) = ws_receiver.next().await {
                match result {
                    Ok(Message::Text(text)) => {
                        println!("[SignalingManager] Received text message: {}", text);
                        if let Ok(json_msg) = serde_json::from_str::<Value>(&text) {
                            // Process SDP answer.
                            if let Some(answer) = json_msg.get("answer").and_then(|v| v.as_str()) {
                                println!("Received SDP answer.\nPeer connection: {:?}", peer_connection);
                                match RTCSessionDescription::answer(answer.to_string()) {
                                    Ok(desc) => {
                                        println!("Sending answer: {:?}", desc);
                                        if let Err(e) = peer_connection.set_remote_description(desc).await {
                                            println!("Error setting remote descriptionxxxxx: {:?}", e);
                                        } else {
                                            println!("Set remote description successfully");
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error parsing SDP answer: {:?}", e);
                                    }
                                }
                            }

                            // Process ICE candidate.
                            if let Some(candidate) = json_msg.get("candidate") {
                                println!("Received ICE candidate: {:?}", candidate);
                                let candidate_str = serde_json::to_string(candidate).unwrap_or_default();
                                match serde_json::from_str::<RTCIceCandidateInit>(&candidate_str) {
                                    Ok(candidate_init) => {
                                        if let Err(e) = peer_connection.add_ice_candidate(candidate_init).await {
                                            println!("Error adding ICE candidate: {:?}", e);
                                        }
                                    }
                                    Err(e) => {
                                        println!("Error parsing ICE candidate: {:?}", e);
                                    }
                                }
                            }

                            // Process active_clients message.
                            if json_msg.get("type") == Some(&json!("active_clients")) {
                                println!("[SignalingManager] Received active_clients message.");
                                if let Some(clients) = json_msg.get("clients").and_then(|v| v.as_array()) {
                                    if clients.iter().any(|c| c.get("role") == Some(&json!("answerer"))) {
                                        println!("[SignalingManager] Answerer joined, resending offer...");
                                        let pc = peer_connection.clone();
                                        if let Some(local_desc) = pc.local_description().await {
                                            let offer_msg = json!({
                                                "offer": local_desc.sdp,
                                                "join_code": "room1"
                                            })
                                            .to_string();
                                            if let Err(e) = ws_sender.lock().await.send(Message::Text(offer_msg.into())).await {
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
                        // Respond to pings.
                        println!("[SignalingManager] Received Ping, sending Pong");
                        if let Err(e) = ws_sender.lock().await.send(Message::Pong(data)).await {
                            println!("Error sending Pong: {:?}", e);
                        }
                    }
                    Ok(Message::Pong(_)) => {
                        println!("[SignalingManager] Received Pong");
                    }
                    Ok(Message::Close(_)) => {
                        println!("[SignalingManager] WebSocket closed by remote");
                        break;
                    }
                    Ok(other) => {
                        println!("[SignalingManager] Received non-text message: {:?}", other);
                    }
                    Err(e) => {
                        println!("[SignalingManager] WebSocket error: {:?}", e);
                        break;
                    }
                }
            }
            println!("[SignalingManager] Receiver loop ended");
        });
    }
}
