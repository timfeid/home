use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use reqwest::header::CONTENT_DISPOSITION;
use reqwest::Url;
use reqwest::{header::CONTENT_LENGTH, Client};
use tokio::task::JoinHandle;

#[derive(Debug)]
struct Part {
    url: String,
    from: u64,
    to: u64,
    content_length: u64,
    downloaded: u64,
    file: File,
    file_path: PathBuf,
}

const TOTAL_PARTS: usize = 8;

impl Part {
    fn to(part_num: &usize, content_length: &u64) -> u64 {
        if *part_num == TOTAL_PARTS - 1 {
            return content_length - 1;
        }

        ((*part_num as u64) + 1) * Part::equal_bytes(content_length) - 1
    }
    fn from(part_num: &usize, content_length: &u64) -> u64 {
        (*part_num as u64) * Part::equal_bytes(content_length)
    }

    fn equal_bytes(content_length: &u64) -> u64 {
        content_length / (TOTAL_PARTS as u64)
    }

    pub fn new(url: &str, part_num: &usize, content_length: &u64, temp_dir: &PathBuf) -> Self {
        let from = Part::from(&part_num, &content_length);
        let to = Part::to(&part_num, &content_length);
        let file_path = temp_dir.join(format!("test.part.{}", part_num));

        Part {
            url: url.to_string(),
            content_length: to - from + 1,
            from,
            to,
            downloaded: 0,
            file: File::create(&file_path.as_path()).expect("unable to create file"),
            file_path,
        }
    }
}

async fn download_part(part: Arc<Mutex<Part>>, client: Client) -> Result<()> {
    let range_header = {
        let part = part.lock().unwrap();
        format!("bytes={}-{}", part.from, part.to)
    };

    let response = client
        .get(&{
            let part = part.lock().unwrap();
            part.url.clone()
        })
        .header("Range", range_header)
        .send()
        .await?;

    let content = response.bytes().await?;

    let mut part = part.lock().unwrap();
    part.downloaded += content.len() as u64;
    part.file.write_all(&content)?;

    println!(
        "{} is {}% complete",
        part.file_path.as_path().display(),
        part.downloaded / part.content_length * 100,
    );

    Ok(())
}

async fn download_in_chunks(url: &str, filename: &str, client: Client, length: u64) -> Result<()> {
    let temp_dir = PathBuf::from("temp_download_dir");
    fs::create_dir_all(&temp_dir)?;

    let parts: Vec<Arc<Mutex<Part>>> = (0..TOTAL_PARTS)
        .map(|i| Arc::new(Mutex::new(Part::new(url, &i, &length, &temp_dir))))
        .collect();
    println!("parts: {:?}", parts);

    let mut handles: Vec<JoinHandle<Result<()>>> = vec![];

    for part in parts.iter() {
        let client = client.clone();
        let part = Arc::clone(part);
        let handle = tokio::spawn(async move {
            if let Err(e) = download_part(part, client).await {
                eprintln!("Failed to download part: {:?}", e);
            }
            Ok(())
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await??;
    }

    let output_file_path = PathBuf::from("temp_download_dir/".to_owned() + filename);
    let mut output_file = File::create(&output_file_path)?;

    for part in parts.iter() {
        let part = part.lock().unwrap();
        let mut part_file = File::open(&part.file_path)?;
        let mut buffer = Vec::new();
        part_file.read_to_end(&mut buffer)?;
        output_file.write_all(&buffer)?;
    }

    println!("Combined file created at {:?}", output_file_path);

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
        .unwrap_or_else(|| "filename.zip".to_string())
}

async fn download(url: String, client: Client) -> Result<()> {
    let response = client.head(&url).send().await?;
    println!("response: {:?}", response);

    let filename = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|header| extract_filename_from_header(header))
        .unwrap_or_else(|| derive_filename_from_url(&url));

    if let Some(length) = response.headers().get(CONTENT_LENGTH) {
        return download_in_chunks(
            &url,
            &filename,
            client,
            length
                .to_str()
                .map_err(|_| anyhow!("unable to convert header to str"))?
                .parse::<u64>()
                .map_err(|_| anyhow!("unable to convert header to u64"))?,
        )
        .await;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = reqwest::Client::new();
    if download(
        "https://dek691o7llrph.cloudfront.net/game/first-update.mp4".to_string(),
        client,
    )
    .await
    .is_ok()
    {
        println!("done");
    }

    Ok(())
}
