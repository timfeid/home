use anyhow::Result;
use std::{path::PathBuf, sync::Arc};
use tauri::{AppHandle, Manager};
use tauri_plugin_store::{Store, StoreExt};

pub fn get_app_data_path<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .expect("Unable to find app data directory")
}

pub fn get_store<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> Result<Arc<Store<R>>> {
    let path = get_app_data_path(app_handle);
    let store = app_handle.store(path)?;

    Ok(store)
}

// Audio utility functions that could be shared across modules
pub fn float_to_i16_samples(float_samples: &[f32]) -> Vec<i16> {
    float_samples
        .iter()
        .map(|&s| (s * 32767.0) as i16)
        .collect()
}

pub fn i16_to_float_samples(i16_samples: &[i16]) -> Vec<f32> {
    i16_samples.iter().map(|&s| s as f32 / 32767.0).collect()
}

pub fn log_error<T, E: std::fmt::Debug>(result: Result<T, E>, message: &str) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(err) => {
            eprintln!("{}: {:?}", message, err);
            None
        }
    }
}
