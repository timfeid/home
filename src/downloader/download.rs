use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use log::{error, info};
use reqwest::{
    header::{CONTENT_DISPOSITION, CONTENT_LENGTH},
    Client, Url,
};
use tokio::task::JoinHandle;
use tokio::time::{sleep, Duration};

const DEFAULT_PARTS: usize = 8;
const MAX_RETRY: u8 = 3;

struct DownloadTask {
    url: String,
    filename: String,
    total_size: u64,
    total_parts: usize,
    client: Client,
    output_path: PathBuf,
}

impl DownloadTask {
    fn new(
        url: String,
        filename: String,
        total_size: u64,
        total_parts: usize,
        client: Client,
        download_path: PathBuf,
    ) -> Self {
        let output_path = download_path.join(&filename);
        DownloadTask {
            url,
            filename,
            total_size,
            total_parts,
            client,
            output_path,
        }
    }

    async fn execute(&self) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.output_path)?;
        file.set_len(self.total_size)?;
        let file = Arc::new(file);

        let mut handles: Vec<JoinHandle<Result<()>>> = Vec::new();

        for part_num in 0..self.total_parts {
            let client_clone = self.client.clone();
            let file_clone = file.clone();
            let url = self.url.clone();
            let total_size = self.total_size;
            let total_parts = self.total_parts;

            let handle = tokio::spawn(async move {
                let chunk_size = total_size / total_parts as u64;
                let from = part_num as u64 * chunk_size;
                let to = if part_num == total_parts - 1 {
                    total_size - 1
                } else {
                    (part_num as u64 + 1) * chunk_size - 1
                };

                let range_header = format!("bytes={}-{}", from, to);
                info!("Downloading part {} with range: {}", part_num, range_header);

                let mut retry_count = 0;
                loop {
                    let response = client_clone
                        .get(&url)
                        .header("Range", &range_header)
                        .send()
                        .await;

                    match response {
                        Ok(resp) => {
                            if resp.status().is_success() || resp.status().as_u16() == 206 {
                                let content = resp.bytes().await?;

                                let mut file_handle = file_clone.try_clone()?;
                                let len = content.len();
                                tokio::task::spawn_blocking(move || -> Result<()> {
                                    file_handle.seek(SeekFrom::Start(from))?;
                                    file_handle.write_all(&content)?;
                                    Ok(())
                                })
                                .await??;
                                info!("Completed part {}: {} bytes written", part_num, len);
                                break;
                            } else {
                                error!("HTTP error in part {}: {}", part_num, resp.status());
                            }
                        }
                        Err(e) => {
                            error!("Network error in part {}: {:?}", part_num, e);
                        }
                    }
                    retry_count += 1;
                    if retry_count > MAX_RETRY {
                        return Err(anyhow!(
                            "Failed to download part {} after {} retries",
                            part_num,
                            MAX_RETRY
                        ));
                    }
                    info!("Retrying part {} (attempt {})", part_num, retry_count);
                    sleep(Duration::from_secs(2)).await;
                }
                Ok(())
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await??;
        }

        info!("Download complete: {:?}", self.output_path);
        Ok(())
    }
}

pub async fn download(url: String, client: Client, download_path: PathBuf) -> Result<()> {
    info!("d path: {}", download_path.display());
    let head_resp = client.head(&url).send().await?;
    let total_size = if let Some(cl) = head_resp.headers().get(CONTENT_LENGTH) {
        cl.to_str()?.parse::<u64>()?
    } else {
        return Err(anyhow!("Missing Content-Length header"));
    };

    let filename = head_resp
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|header| extract_filename_from_header(header))
        .unwrap_or_else(|| derive_filename_from_url(&url));

    info!("Starting download: {} ({} bytes)", filename, total_size);

    let task = DownloadTask::new(
        url,
        filename,
        total_size,
        DEFAULT_PARTS,
        client,
        download_path,
    );
    task.execute().await?;
    Ok(())
}

fn extract_filename_from_header(header_value: &str) -> Option<String> {
    header_value.split(';').find_map(|part| {
        let part = part.trim();
        if part.starts_with("filename=") {
            Some(part["filename=".len()..].trim_matches('"').to_string())
        } else {
            None
        }
    })
}

fn derive_filename_from_url(url: &str) -> String {
    Url::parse(url)
        .ok()
        .and_then(|parsed_url| {
            parsed_url
                .path_segments()
                .and_then(|segments| segments.last())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "downloaded_file".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use md5;
    use std::fs::File;
    use std::io::Read;
    use tokio::fs;

    #[tokio::test]
    async fn test_download_checksum() -> Result<()> {
        let download_url = "https://dek691o7llrph.cloudfront.net/game/first-update.mp4".to_string();
        let client = Client::new();

        let filename = derive_filename_from_url(&download_url);

        // let _ = fs::remove_file(&filename).await;

        let dir = std::env::temp_dir();
        download(download_url, client, dir.clone()).await?;

        let mut file = File::open(dir.join(&filename))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let digest = format!("{:x}", md5::compute(&buffer));

        assert_eq!(
            digest, "655b910d3f0141b49e3baefb53dbe11e",
            "MD5 checksum mismatch: expected 655b910d3f0141b49e3baefb53dbe11e, got {}",
            digest
        );
        let _ = fs::remove_file(&filename).await;
        Ok(())
    }
}
