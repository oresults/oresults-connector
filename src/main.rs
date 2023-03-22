use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;
use anyhow::{anyhow, Error};
use clap::{Parser};
use flate2::Compression;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{error, info, Level, warn};
use tracing_subscriber::FmtSubscriber;
use flate2::bufread::ZlibEncoder;
use reqwest::blocking::multipart::Part;

const API_URL: &str = "https://api.oresults.eu";

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// API key provided in event settings
    #[clap(short, long, value_name = "API_KEY")]
    key: String,

    /// Path to folder (or a single file) recursively watched for changes
    #[clap(short, long, value_name = "PATH_TO_FILES")]
    path: Option<String>,

}

fn get_file_name(path: PathBuf) -> Result<String, Error>{
    if let Some(file_name) = path.file_name() {
        let file_name = file_name.to_str().ok_or_else(|| anyhow!("Invalid characters in file_name"))?.to_string();
        return Ok(file_name);
    }
    Err(anyhow!("Failed to get file_name"))
}

fn is_xml_file(path: &PathBuf, name: &[u8]) -> bool {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) => {
            error!("Failed to open file: {}", e);
            return false;
        }
    };
    let mut buf = Vec::new();
    let r = BufReader::new(file);
    let mut reader = quick_xml::reader::Reader::from_reader(r);
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Eof) | Err(_) => break,
            Ok(quick_xml::events::Event::Start(e)) => {
                return e.name().into_inner() == name;
            }
            _ => (),
        }
    };
    false
}

/// looks for pattern found in IOF XML v3 StartList
fn is_start_list(path: &PathBuf) -> bool {
    is_xml_file(path, b"StartList")
}

/// looks for pattern found in IOF XML v3 ResultList
fn is_result_list(path: &PathBuf) -> bool {
    is_xml_file(path, b"ResultList")
}

fn compress_file(path: &PathBuf) -> Result<Vec<u8>, String> {
    let file = File::open(path)
        .map_err(|e| format!("Failed to open file: {:?}", e))?;
    let mut compressed = ZlibEncoder::new(BufReader::new(file), Compression::fast());
    let mut buffer = Vec::new();
    compressed.read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file: {:?}", e))?;
    Ok(buffer)
}
fn upload_file(path: &PathBuf, api_key: &str, file_name: &str, file_type: &str, upload_path: &str) {
    let client = reqwest::blocking::Client::new();

    let compressed_file = match compress_file(path) {
        Ok(f) => f,
        Err(e) => {
            error!("{} | {} | {}", file_type, file_name, e);
            return;
        }
    };

    let form = reqwest::blocking::multipart::Form::new()
        .text("apiKey", api_key.to_string())
        .part("file", Part::bytes(compressed_file));

    match client.post(format!("{}{}", API_URL, upload_path))
        .multipart(form)
        .send()
    {
        Ok(response) => {
            if response.status().is_success() {
                info!("{} | {} | uploaded", file_type, file_name);
            }
            else {
                error!("{} | {} | upload failed, {}", file_type, file_name, response.text().unwrap_or_else(|_| "cannot decode response body".to_string()))
            }
        },
        Err(e) => error!("{} | {} | upload failed, {}", file_type, file_name, e)
    };
}

fn watch_and_upload(folder_path: PathBuf, api_key: &str) {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(100))
        .expect("Failed to initialize watcher");
    loop {
        if watcher.watch(&folder_path, RecursiveMode::Recursive).is_err() {
            let path = if folder_path.is_absolute() {
                folder_path.clone()
            }
            else {
                PathBuf::from(".").canonicalize().unwrap_or_else(|_| PathBuf::from(".")).join(folder_path.clone())
            };
            error!("Path {:?} does not exist", path);
            sleep(Duration::from_secs(10));
            continue;
        }
        info!("{:?} is a valid path, recursively watching for file changes", folder_path.canonicalize().unwrap_or(folder_path));
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Create(p)) |
                Ok(DebouncedEvent::Write(p)) |
                Ok(DebouncedEvent::Chmod(p)) => {
                    if let Ok(file_name) = get_file_name(p.clone()) {
                         if is_start_list(&p) {
                             info!("StartList | {} | change detected", file_name);
                             upload_file(&p, api_key, &file_name, "StartList",  "/start-lists");
                         }
                         else if is_result_list(&p) {
                             info!("Results | {} | change detected", file_name);
                             upload_file(&p, api_key, &file_name, "Results", "/results");
                         }
                         else {
                             warn!("unrecognized | {:?} | change detected", p.canonicalize().unwrap_or(p));
                         }
                    }
                },
                Ok(DebouncedEvent::NoticeRemove(_)) |
                Ok(DebouncedEvent::Remove(_)) |
                Ok(DebouncedEvent::NoticeWrite(_)) |
                Ok(DebouncedEvent::Rename(_,_)) |
                Ok(DebouncedEvent::Rescan)
                    => {},
                Ok(DebouncedEvent::Error(e, _)) => println!("watch error: {:?}", e),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let path = PathBuf::from(cli.path.unwrap_or_else(|| ".".to_string()));

    watch_and_upload(path, &cli.key)
}