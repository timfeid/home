// modules/config.rs
use serde::{Deserialize, Serialize};

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

impl Default for AudioCaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            buffer_size: 1024,
            capture_mode: CaptureMode::VoiceActivated,
            voice_activity_threshold: -40.0,
        }
    }
}
