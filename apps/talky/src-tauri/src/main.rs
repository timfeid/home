#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Duration;

use anyhow::Context;
use app_lib::run;
use cpal::traits::{DeviceTrait, HostTrait};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run();

    Ok(())
}
