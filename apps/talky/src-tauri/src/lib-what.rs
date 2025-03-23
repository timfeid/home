mod recording;
mod rtc;
mod signal;

use recording::AudioCommand;
use serde_json::json;
use signal::SignalingManager;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::{mpsc, Mutex};
use tungstenite::Message;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::track::track_local::TrackLocal;

pub struct AudioRecorderManager {
    command_sender: mpsc::Sender<AudioCommand>,
    peer_connection: Arc<webrtc::peer_connection::RTCPeerConnection>,
    audio_track: Arc<webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample>,
    signaling: Arc<Mutex<SignalingManager>>,
}

impl AudioRecorderManager {
    pub async fn new() -> Self {
        // Create a channel for audio commands.
        let (tx, rx) = mpsc::channel::<AudioCommand>(10);

        println!("Initializing RTC components");
        let (peer_connection, audio_track) = rtc::create_peer_connection().await;

        // Set up a peer connection state change handler.
        let _pc_state = Arc::clone(&peer_connection);

        let signaling_manager = Arc::new(Mutex::new(
            SignalingManager::new(Arc::clone(&peer_connection)).unwrap(),
        ));
        let signaling_manager_clone = signaling_manager.clone();

        peer_connection.on_ice_candidate(Box::new(move |candidate| {
            let signal_manager = signaling_manager_clone.clone();
            Box::pin(async move {
                signal_manager
                    .lock()
                    .await
                    .on_ice_candidate(candidate)
                    .await
            })
        }));

        rtc::create_offer(peer_connection.clone()).await;
        if let Some(local_desc) = peer_connection.local_description().await {
            let offer_msg = json!({
                "offer": local_desc.sdp,
                "join_code": "room1"
            })
            .to_string();

            signaling_manager.lock().await.send(offer_msg).await;
        } else {
            println!("Failed to get local description!");
        }

        // Spawn the audio thread on a dedicated thread.
        let audio_track_clone = Arc::clone(&audio_track);
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(recording::audio_thread(rx, audio_track_clone));
        });

        Self {
            command_sender: tx,
            peer_connection,
            audio_track,
            signaling: signaling_manager,
        }
    }

    pub async fn start(&self) {
        println!("Sending Start command");
        let _ = self.command_sender.send(AudioCommand::Start).await;
    }

    pub async fn stop(&self) {
        println!("Sending Stop command");
        let _ = self.command_sender.send(AudioCommand::Stop).await;
    }
}

#[tauri::command]
async fn start_record(state: State<'_, AudioRecorderManager>) -> Result<(), String> {
    println!("Tauri command: start_record");
    state.start().await;
    Ok(())
}

#[tauri::command]
async fn end_record(state: State<'_, AudioRecorderManager>) -> Result<(), String> {
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
                    println!("🚀 Initializing AudioRecorderManager in background...");
                    let manager = AudioRecorderManager::new().await;
                    println!("✅ AudioRecorderManager ready!");
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
