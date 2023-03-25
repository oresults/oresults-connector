use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use flate2::bufread::ZlibEncoder;
use flate2::Compression;
use reqwest::blocking::multipart::Part;
use tracing::error;
use crate::API_URL;

#[derive(Debug)]
pub enum FileType {
    StartList,
    ResultList,
}

pub fn get_file_type(path: &PathBuf) -> Option<FileType> {
    if is_start_list(path) {
        Some(FileType::StartList)
    } else if is_result_list(path) {
        Some(FileType::ResultList)
    } else {
        None
    }
}

pub fn upload_file(path: &PathBuf, api_key: &str, upload_path: &str) -> Result<(), String> {
    let client = reqwest::blocking::Client::new();

    let compressed_file = match compress_file(path) {
        Ok(f) => f,
        Err(e) => return Err(format!("{:?}", e))
    };

    let form = reqwest::blocking::multipart::Form::new()
        .text("apiKey", api_key.to_owned())
        .part("file", Part::bytes(compressed_file));

    match client.post(format!("{}{}", API_URL, upload_path))
        .multipart(form)
        .send()
    {
        Ok(response) => {
            if response.status().is_success() {
                Ok(())
            }
            else {
                Err(response.text().unwrap_or_else(|_| "cannot decode response body".to_string()))
            }
        },
        Err(e) => Err(e.to_string())
    }
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