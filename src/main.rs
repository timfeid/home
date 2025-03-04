use anyhow::Result;
use downloader::manager::DownloadManager;
use reqwest::Client;

mod downloader;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let download_url = "https://dek691o7llrph.cloudfront.net/game/first-update.mp4".to_string();
    let client = Client::new();
    // if let Err(e) = download(download_url, client).await {
    //     error!("Download failed: {:?}", e);
    // }
    let mut manager = DownloadManager::new(8, client);
    // manager
    //     .set_download_path_str("./temp_download_dir".to_string())
    //     .await;
    manager.set_download_path(std::env::temp_dir()).await;
    manager.add_download(download_url).await?;
    manager.wait_all().await;
    Ok(())
}
