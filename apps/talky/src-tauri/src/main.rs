// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use app_lib::{run, AudioCaptureConfig, CaptureMode, WebRTCManager};

fn default_audio_config() -> AudioCaptureConfig {
    AudioCaptureConfig {
        sample_rate: 48000,
        channels: 2,
        buffer_size: 1024,
        capture_mode: CaptureMode::PushToTalk,
        voice_activity_threshold: -40.0, // dB threshold
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Define the default audio capture configuration

    let default_config = AudioCaptureConfig {
        sample_rate: 48000,
        channels: 2,
        buffer_size: 1024,
        capture_mode: CaptureMode::VoiceActivated,
        voice_activity_threshold: -40.0, // in dB
    };

    // Initialize WebRTCManager
    let webrtc_manager = WebRTCManager::new(
        default_config,
        "ws://localhost:8080/soundhouse",
        "room1".to_string(),
    )
    .await
    .expect("Failed to initialize WebRTC");

    // Start audio capture
    let stream = webrtc_manager
        .start_audio_capture()
        .await
        .expect("Failed to start audio capture");

    // Hold onto the stream so it’s not dropped
    let _audio_stream_handle = stream.clone();

    // Run Tauri with the manager
    run(webrtc_manager);

    Ok(())
}
