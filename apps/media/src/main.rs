use anyhow::Result;
use downloader::manager::DownloadManager;
use reqwest::Client;
use thirtyfour::prelude::*;

mod downloader;

async fn test() -> WebDriverResult<()> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:53290", caps).await?;

    // Navigate to https://wikipedia.org.
    driver.goto("https://wikipedia.org").await?;
    let elem_form = driver.find(By::Id("search-form")).await?;

    // Find element from element.
    let elem_text = elem_form.find(By::Id("searchInput")).await?;

    // Type in the search terms.
    elem_text.send_keys("selenium").await?;

    // Click the search button.
    let elem_button = elem_form.find(By::Css("button[type='submit']")).await?;
    elem_button.click().await?;

    // Look for header to implicitly wait for the page to load.
    driver.find(By::ClassName("firstHeading")).await?;
    assert_eq!(driver.title().await?, "Selenium - Wikipedia");

    // Always explicitly close the browser.
    driver.quit().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    // let download_url = "https://dek691o7llrph.cloudfront.net/game/first-update.mp4".to_string();
    // let client = Client::new();
    // // if let Err(e) = download(download_url, client).await {
    // //     error!("Download failed: {:?}", e);
    // // }
    // let mut manager = DownloadManager::new(8, client);
    // // manager
    // //     .set_download_path_str("./temp_download_dir".to_string())
    // //     .await;
    // manager.set_download_path(std::env::temp_dir()).await;
    // manager.add_download(download_url).await?;
    // manager.wait_all().await;
    test().await?;
    Ok(())
}
