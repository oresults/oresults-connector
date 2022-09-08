use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::Duration;
use anyhow::{anyhow, Error};
use clap::{Parser};
use lazy_regex::regex_is_match;
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

    /// Path to folder (or a single file) recursively watched for changes
    #[clap(short, long, value_name = "PATH_TO_FILES")]
    path: Option<String>,

}

fn upload_file(file: String, api_key: &String, file_name: &str, upload_path: &str) {
    let client = reqwest::blocking::Client::new();

    let form = reqwest::blocking::multipart::Form::new()
        .text("apiKey", api_key.clone())
        .text("file", file);

    match client.post(format!("{}{}", API_URL, upload_path))
        .multipart(form)
        .send()
    {
        Ok(response) => {
            if response.status().is_success() {
                info!("\"{}\" uploaded", file_name);
            }
            else {
                error!("\"{}\" upload failed: {:?}", file_name, response.text().unwrap_or("cannot decode response body".to_string()))
            }
        },
        Err(e) => error!("\"{}\" upload failed: {:?}", file_name, e)
    };
}

fn watch_and_upload(folder_path: PathBuf, api_key: String) {
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(100))
        .expect("Failed to initialize watcher");
    loop {
        if watcher.watch(&folder_path, RecursiveMode::Recursive).is_err() {
            let path = if folder_path.is_absolute() {
                folder_path.clone()
            }
            else {
                PathBuf::from(".").canonicalize().unwrap_or(PathBuf::from(".")).join(folder_path.clone())
            };
            error!("Path {:?} does not exist", path);
            sleep(Duration::from_secs(10));
            continue;
        }
        info!("{:?} is a valid path, recursively watching for file changes", folder_path.canonicalize().unwrap_or(folder_path));
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::NoticeRemove(p)) => {
                    if let Ok((file, file_name)) = get_xml_file_as_string(p) {
                        if is_start_list(&file) {
                            info!("StartList file removed: {}", &file_name);
                        }
                        if is_result_list(&file) {
                            info!("ResultList file removed: {}", &file_name);
                        }
                    }
                },
                Ok(DebouncedEvent::Create(p)) |
                Ok(DebouncedEvent::Write(p)) |
                Ok(DebouncedEvent::Chmod(p)) => {
                    if let Ok((file, file_name)) = get_xml_file_as_string(p.clone()) {
                        if is_start_list(&file) {
                            info!("StartList file changed");
                            upload_file(file, &api_key, &file_name, "/start-lists?format=xml");
                        }
                        else if is_result_list(&file) {
                            info!("ResultList file changed");
                            upload_file(file, &api_key, &file_name, "/results");
                        }
                    }
                    else {
                        warn!("File changed, but not recognized: {:?}", p.canonicalize().unwrap_or(p));
                    }
                },
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

fn get_xml_file_as_string(path: PathBuf) -> Result<(String, String), Error> {
    if path.extension() == Some(OsStr::new("xml")) {
        if let Some(file_name) = path.file_name() {
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            let file_name = file_name.to_str().ok_or(anyhow!("Could not get file_name"))?.to_string();
            return Ok((contents, file_name));
        }
    }
    Err(anyhow!("Failed to convert file to xml string"))
}


fn main() {
    let cli = Cli::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    let path = PathBuf::from(cli.path.unwrap_or(".".to_string()));

    watch_and_upload(path, cli.key)

}


/// looks for pattern found in IOF XML v3 StartList
fn is_start_list(file: &str) -> bool {
    return regex_is_match!(r"<StartList[\s\S]*</StartList>", file);
}

/// looks for pattern found in IOF XML v3 ResultList
fn is_result_list(file: &str) -> bool {
    return regex_is_match!(r"<ResultList[\s\S]*</ResultList>", file);
}