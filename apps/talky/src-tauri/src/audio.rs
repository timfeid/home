use crate::config::{AudioCaptureConfig, CaptureMode};
use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, StreamConfig};
use opus::{Application, Channels, Encoder};
use std::cell::RefCell;
use std::sync::Arc;
use std::thread_local;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

thread_local! {
    pub static AUDIO_STREAM: RefCell<Option<cpal::Stream>> = RefCell::new(None);
    pub static FRONTEND_AUDIO_STREAM: RefCell<Option<cpal::Stream>> = RefCell::new(None);
}

pub struct AudioProcessor {
    config: AudioCaptureConfig,
    opus_encoder: Arc<Mutex<Encoder>>,
}

pub struct CaptureControl {
    push_to_talk_active: bool,
    frontend_monitoring: bool,
}

impl CaptureControl {
    pub fn new() -> Self {
        Self {
            push_to_talk_active: false,
            frontend_monitoring: false,
        }
    }

    pub fn set_push_to_talk(&mut self, active: bool) {
        self.push_to_talk_active = active;
    }

    pub fn is_push_to_talk_active(&self) -> bool {
        self.push_to_talk_active
    }

    pub fn set_frontend_monitoring(&mut self, active: bool) {
        self.frontend_monitoring = active;
    }

    pub fn is_frontend_monitoring(&self) -> bool {
        self.frontend_monitoring
    }
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
                println!("{}", db_level);
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

#[derive(Clone, serde::Serialize)]
pub struct AudioDataPayload {
    level: f32,       // in dB
    rms: f32,         // raw RMS value
    sample_rate: u32, // sample rate
    channels: u16,    // number of channels
}

pub fn setup_audio_device() -> Result<(cpal::Device, StreamConfig)> {
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
    Ok((input_device, stream_config))
}

pub fn start_frontend_monitoring(app_handle: AppHandle) -> Result<()> {
    println!("[AudioCapture] Initializing frontend monitoring...");

    let (input_device, stream_config) = setup_audio_device()?;
    let channels = stream_config.channels as usize;

    let err_fn = move |err| {
        eprintln!("[FrontendAudio] Error in stream: {}", err);
    };

    let app_handle_clone = app_handle.clone();

    let stream = input_device.build_input_stream(
        &stream_config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // Process audio data for visualization
            let rms = AudioProcessor::calculate_rms(data);
            let db_level = AudioProcessor::rms_to_decibels(rms);

            // Send audio data to frontend
            let payload = AudioDataPayload {
                level: db_level,
                rms,
                sample_rate: stream_config.sample_rate.0,
                channels: channels as u16,
            };

            if let Err(e) = app_handle_clone.emit("microphone-data", payload) {
                eprintln!("[FrontendAudio] Failed to emit microphone data: {:?}", e);
            }
        },
        err_fn,
        None,
    )?;

    stream.play()?;

    FRONTEND_AUDIO_STREAM.with(|stream_cell| {
        *stream_cell.borrow_mut() = Some(stream);
    });

    println!("[AudioCapture] Frontend monitoring stream started");
    Ok(())
}

pub fn stop_frontend_monitoring() -> Result<()> {
    FRONTEND_AUDIO_STREAM.with(|stream_cell| {
        if let Some(stream) = stream_cell.borrow_mut().take() {
            drop(stream);
            println!("[AudioCapture] Frontend monitoring stream stopped");
        }
    });
    Ok(())
}
