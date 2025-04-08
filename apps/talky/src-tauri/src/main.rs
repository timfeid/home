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

    let webrtc_manager = WebRTCManager::new("ws://localhost:8080/soundhouse")
        .await
        .expect("Failed to initialize WebRTC");

    run(webrtc_manager);

    Ok(())
}
