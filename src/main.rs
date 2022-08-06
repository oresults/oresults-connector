use std::{fs};
use std::sync::mpsc::{channel};
use std::thread::{sleep, spawn};
use std::time::Duration;
use clap::{Parser};
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use tracing::{error, info, Level, warn};
use tracing_subscriber::FmtSubscriber;

const API_URL: &'static str = "https://api.oresults.eu";

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// API key provided in event settings
    #[clap(short, long, value_name = "API_KEY")]
    key: String,

    /// Path to iof xml v3 StartList file
    #[clap(short, long, value_name = "PATH_STARTLIST")]
    startlist: Option<String>,

    /// Path to iof xml v3 ResultList file
    #[clap(short, long, value_name = "PATH_RESULTS")]
    results: String,
}

fn upload_file(path: &String, api_key: &String, file_name: &str, upload_path: &str) {
    let client = reqwest::blocking::Client::new();

    let file = match fs::read_to_string(path) {
        Ok(file) => file,
        Err(e) => {
            error!("{} upload failed: {:?}", file_name, e);
            return;
        }
    };

    let form = reqwest::blocking::multipart::Form::new()
        .text("apiKey", api_key.clone())
        .text("file", file);

    match client.post(format!("{}{}", API_URL, upload_path))
        .multipart(form)
        .send()
    {
        Ok(response) => {
            if response.status().is_success() {
                info!("{} uploaded", file_name);
            }
            else {
                error!("{} upload failed: {:?}", file_name, response.text().unwrap_or("cannot decode response body".to_string()))
            }
        },
        Err(e) => error!("{} upload failed:: {:?}", file_name, e)
    };
}

fn watch_and_upload(path: String, api_key: String, file_name: String, upload_path: String) {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(100))
        .expect("Failed to initialize watcher");
    loop {
        if watcher.watch(&path, RecursiveMode::Recursive).is_err() {
            warn!("{} file missing: {}", file_name, &path);
            sleep(Duration::from_secs(10));
            continue;
        }
        info!("{} file found", file_name);
        upload_file(&path, &api_key, &file_name, &upload_path);
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::NoticeRemove(_)) => {
                    info!("{} file removed", file_name);
                    break;
                }
                Ok(DebouncedEvent::Remove(_)) |
                Ok(DebouncedEvent::NoticeWrite(_)) => {},
                Ok(_) => {
                    info!("{} file changed", file_name);
                    upload_file(&path, &api_key, &file_name, &upload_path);
                },
                Err(e) => println!("watch error: {:?}", e)
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

    let key = cli.key.clone();

    // results
    let thread_handle = spawn(move ||
        watch_and_upload(cli.results, cli.key, "Results".to_string(), "/results".to_string())
    );

    // startlist
    if let Some(startlist_path) = cli.startlist {
        watch_and_upload(startlist_path, key, "Startlist".to_string(), "/start-lists?format=xml".to_string());
    }

    thread_handle.join().unwrap();
}
