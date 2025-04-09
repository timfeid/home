// modules/commands.rs
use anyhow::Result;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::audio::{start_frontend_monitoring, stop_frontend_monitoring};
use crate::config::{AudioCaptureConfig, CaptureMode};
use crate::webrtc::WebRTCManager;

#[tauri::command]
pub async fn connect_audio(
    auth_token: String,
    channel_id: String,
    niche_id: String,
    state: tauri::State<'_, Arc<WebRTCManager>>,
) -> Result<(), String> {
    println!(
        "[ConnectAudio] Starting with auth_token: {}, channel_id: {}, niche_id: {}",
        auth_token, channel_id, niche_id
    );

    let config = AudioCaptureConfig {
        sample_rate: 48000,
        channels: 2,
        buffer_size: 1024,
        capture_mode: CaptureMode::VoiceActivated,
        voice_activity_threshold: -40.0,
    };

    let state_clone = state.inner().clone();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = state_clone
            .start(config, auth_token, channel_id.clone(), niche_id.clone())
            .await
        {
            eprintln!("[ConnectAudio] Failed to start WebRTC session: {e}");
            return;
        }

        if let Err(e) = state_clone.start_audio_capture().await {
            eprintln!("[ConnectAudio] Failed to start audio capture: {e}");
        }
    });

    println!("[ConnectAudio] Audio capture request initiated");
    Ok(())
}

#[tauri::command]
pub async fn set_refresh_token(
    app: AppHandle,
    token: String,
) -> Result<(), tauri_plugin_store::Error> {
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
pub async fn get_refresh_token(app: AppHandle) -> Result<Option<Value>, tauri_plugin_store::Error> {
    let path = app
        .path()
        .app_data_dir()
        .expect("unable to find data dir")
        .join("data.json");

    app.store(path).and_then(|s| Ok(s.get("refresh_token")))
}

#[tauri::command]
pub async fn configure_audio_capture(
    _config: AudioCaptureConfig,
    _state: tauri::State<'_, WebRTCManager>,
) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn toggle_push_to_talk(
    active: bool,
    state: tauri::State<'_, WebRTCManager>,
) -> Result<(), String> {
    if let Some(audio) = &*state.audio.read().await {
        let mut capture_control = audio.capture_control.lock().await;

        capture_control.set_push_to_talk(active);

        println!("Push-to-talk {}activated", if active { "" } else { "de" });
    } else {
        println!("Not in channel yet?");
    }

    Ok(())
}

#[tauri::command]
pub async fn settings_microphone(active: bool, app_handle: AppHandle) -> Result<(), String> {
    println!(
        "[SettingsMicrophone] Microphone monitoring: {}",
        if active { "started" } else { "stopped" }
    );

    if active {
        // Start frontend monitoring
        if let Err(e) = start_frontend_monitoring(app_handle) {
            eprintln!(
                "[SettingsMicrophone] Failed to start frontend monitoring: {:?}",
                e
            );
            return Err(format!("Failed to start frontend monitoring: {}", e));
        }
    } else {
        // Stop frontend monitoring
        if let Err(e) = stop_frontend_monitoring() {
            eprintln!(
                "[SettingsMicrophone] Failed to stop frontend monitoring: {:?}",
                e
            );
            return Err(format!("Failed to stop frontend monitoring: {}", e));
        }
    }

    Ok(())
}
