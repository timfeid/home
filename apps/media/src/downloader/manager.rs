use anyhow::Result;
use std::path::{Path, PathBuf};

use log::{error, info};
use reqwest::Client;
use std::sync::Arc;
use tokio::{
    fs,
    sync::{mpsc, Mutex},
};

use super::download::download;

struct DownloadJob {
    url: String,
    directory: PathBuf,
}

pub struct DownloadManager {
    sender: mpsc::Sender<DownloadJob>,
    workers: Vec<tokio::task::JoinHandle<()>>,
    download_directory: PathBuf,
}

impl DownloadManager {
    pub fn new(concurrency: usize, client: Client) -> Self {
        let (tx, rx) = mpsc::channel::<DownloadJob>(100);

        let rx = Arc::new(Mutex::new(rx));
        let mut workers = Vec::with_capacity(concurrency);

        for i in 0..concurrency {
            let rx_clone = rx.clone();
            let client_clone = client.clone();
            let handle = tokio::spawn(async move {
                loop {
                    let job_opt = {
                        let mut rx_lock = rx_clone.lock().await;
                        rx_lock.recv().await
                    };
                    match job_opt {
                        Some(job) => {
                            if let Err(e) =
                                download(job.url, client_clone.clone(), job.directory).await
                            {
                                error!("Worker {}: Download failed: {:?}", i, e);
                            }
                        }

                        None => break,
                    }
                }
            });
            workers.push(handle);
        }

        let download_dir = PathBuf::from(".");

        DownloadManager {
            sender: tx,
            workers,
            download_directory: download_dir,
        }
    }

    pub async fn add_download(&self, url: String) -> Result<()> {
        let job = DownloadJob {
            url,
            directory: self.download_directory.clone(),
        };
        self.sender.send(job).await?;
        Ok(())
    }

    pub async fn wait_all(self) {
        drop(self.sender);
        for worker in self.workers {
            let _ = worker.await;
        }
    }

    pub async fn set_download_path(&mut self, dir: PathBuf) {
        if let Err(e) = fs::create_dir_all(&dir).await {
            eprintln!("Failed to create directory {}: {}", dir.display(), e);
        }
        info!("Set download dir: {}", dir.display());
        self.download_directory = dir;
    }

    pub async fn set_download_path_str(&mut self, path_str: String) {
        let dir = PathBuf::from(&path_str);
        self.set_download_path(dir).await;
    }
}
