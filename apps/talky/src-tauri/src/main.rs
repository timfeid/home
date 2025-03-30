#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use anyhow::Context;
use app_lib::{run, AudioCaptureConfig, CaptureMode, WebRTCManager};
use cpal::traits::{DeviceTrait, HostTrait};

fn default_audio_config() -> AudioCaptureConfig {
    AudioCaptureConfig {
        sample_rate: 48000,
        channels: 2,
        buffer_size: 1024,
        capture_mode: CaptureMode::PushToTalk,
        voice_activity_threshold: -40.0,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let host = cpal::default_host();
    let input_device = host
        .default_input_device()
        .context("No input device available")?;
    let supported_config = input_device.default_input_config()?;
    let default_config = AudioCaptureConfig {
        sample_rate: supported_config.sample_rate().0,
        channels: supported_config.channels(),
        buffer_size: 1024,
        capture_mode: CaptureMode::VoiceActivated,
        voice_activity_threshold: -40.0,
    };

    let webrtc_manager = WebRTCManager::new(
        default_config,
        "ws://localhost:8080/soundhouse",
        "room1".to_string(),
    )
    .await
    .expect("Failed to initialize WebRTC");

    let stream = webrtc_manager
        .start_audio_capture()
        .await
        .expect("Failed to start audio capture");

    let _audio_stream_handle = stream.clone();

    run(webrtc_manager);

    Ok(())
}
