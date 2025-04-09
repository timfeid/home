// modules/mod.rs
mod audio;
mod commands;
mod config;
mod signaling;
mod utils;
mod webrtc;

use std::sync::Arc;
// use tauri::plugin_store::StoreExt;
use tauri::{generate_context, generate_handler, Builder};
use webrtc::WebRTCManager;

pub fn run() {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    let signaling_url = "ws://localhost:8080/soundhouse".to_string();

    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let manager = rt
            .block_on(WebRTCManager::new(&signaling_url))
            .expect("Failed to create WebRTCManager");
        tx.send(manager).expect("Failed to send WebRTCManager");
    });

    let manager = rx.recv().expect("Failed to receive WebRTCManager");
    let manager = Arc::new(manager);

    Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(manager)
        .invoke_handler(generate_handler![
            commands::configure_audio_capture,
            commands::toggle_push_to_talk,
            commands::get_refresh_token,
            commands::set_refresh_token,
            commands::connect_audio,
            commands::settings_microphone,
        ])
        .run(generate_context!())
        .expect("error while running tauri application");
}
