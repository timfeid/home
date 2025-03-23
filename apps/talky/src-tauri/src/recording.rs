use bytemuck::cast_slice;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use opus::{Application, Channels, Encoder};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, Mutex};
use tungstenite::Bytes;
use webrtc::media::Sample;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::track::track_local::TrackLocal;

#[derive(Debug)]
pub enum AudioCommand {
    Start,
    Stop,
}

fn configure_stream(device: &cpal::Device) -> StreamConfig {
    let supported_config = device
        .default_input_config()
        .expect("Failed to get input config");
    let mut config: StreamConfig = supported_config.into();
    config.buffer_size = BufferSize::Fixed(1024);
    config.sample_rate = SampleRate(48000);
    config.channels = 2; // Force stereo
    println!("Configured stream: {:?}", config);
    config
}

pub async fn audio_thread(
    mut rx: mpsc::Receiver<AudioCommand>,
    audio_track: Arc<TrackLocalStaticSample>,
) {
    let mut current_stream: Option<cpal::Stream> = None;
    let gain: f32 = 2.0;

    while let Some(command) = rx.recv().await {
        println!("Received command: {:?}", command);
        match command {
            AudioCommand::Start => {
                let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<f32>>(10);
                if current_stream.is_none() {
                    println!("Starting audio recording");

                    // Use Tokio’s mpsc channel for async compatibility.

                    let host = cpal::default_host();
                    let device = host
                        .default_input_device()
                        .expect("No input device available");
                    println!("Using input device: {}", device.name().unwrap());
                    let config = configure_stream(&device);

                    // Build the input stream.
                    let stream = device
                        .build_input_stream(
                            &config.into(),
                            move |data: &[f32], _| {
                                // Apply gain.
                                let amplified: Vec<f32> = data.iter().map(|s| s * gain).collect();
                                // Since channels are set to 2, no need to duplicate.
                                if let Err(e) = audio_tx.try_send(amplified) {
                                    eprintln!("Error sending audio data: {:?}", e);
                                } else {
                                    println!("Sent {} samples", data.len());
                                }
                            },
                            move |err| {
                                eprintln!("Stream error: {}", err);
                            },
                            None,
                        )
                        .expect("Failed to build input stream");

                    stream.play().expect("Failed to start stream");
                    current_stream = Some(stream);

                    // Create an Opus encoder for stereo (2 channels, 48kHz, VoIP mode).
                    let opus_encoder = Arc::new(Mutex::new(
                        Encoder::new(48000, Channels::Stereo, Application::Voip)
                            .expect("Failed to create Opus encoder"),
                    ));

                    let audio_track_clone = Arc::clone(&audio_track);
                    // Spawn an async task to accumulate, encode, and send audio.
                    tokio::spawn(async move {
                        let mut buffer: Vec<f32> = Vec::new();
                        // For stereo, 20ms equals 960 samples per channel → 1920 total samples.
                        let frame_size = 1920;

                        while let Some(chunk) = audio_rx.recv().await {
                            buffer.extend_from_slice(&chunk);
                            while buffer.len() >= frame_size {
                                let frame: Vec<i16> = buffer
                                    .drain(..frame_size)
                                    .map(|s| (s * 32767.0) as i16)
                                    .collect();

                                let mut opus_buffer = vec![0u8; 4000];
                                let encoded_bytes = {
                                    let mut encoder = opus_encoder.lock().await;
                                    match encoder.encode(&frame, &mut opus_buffer) {
                                        Ok(n) => n,
                                        Err(e) => {
                                            eprintln!("Opus encoding failed: {:?}", e);
                                            continue;
                                        }
                                    }
                                };

                                let sample = Sample {
                                    data: Bytes::copy_from_slice(&opus_buffer[..encoded_bytes]),
                                    timestamp: SystemTime::now(),
                                    duration: Duration::from_millis(20),
                                    packet_timestamp: 0,
                                    prev_dropped_packets: 0,
                                    prev_padding_packets: 0,
                                };

                                if let Err(e) = audio_track_clone.write_sample(&sample).await {
                                    eprintln!("Error sending audio sample: {:?}", e);
                                } else {
                                    println!("✅ Sent {} bytes of Opus", encoded_bytes);
                                }
                            }
                        }
                    });
                }
            }
            AudioCommand::Stop => {
                if current_stream.is_some() {
                    println!("Stopping audio recording");
                    // Dropping the stream stops the recording.
                    current_stream = None;
                }
            }
        }
    }
}
