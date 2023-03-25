#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::Serialize;
use notify_debouncer_mini::{DebounceEventResult, DebounceEventHandler, DebouncedEventKind};
use tauri::{AppHandle, Manager, Wry};
use crate::file_utils::{FileType, get_file_type, upload_file};
use crate::cmd::{pick_folder, set_api_key, watch, unwatch};
use crate::Severity::{Error, Success};
use crate::changes_watcher::ChangesWatcher;

const API_URL: &str = "https://api.oresults.eu";
mod file_utils;
mod cmd;
mod changes_watcher;

#[derive(Clone, Serialize)]
struct LogEntry {
    severity: Severity,
    event: String,
    filename: String
}

#[derive(Clone, Serialize)]
enum Severity {
    Success,
    Error
}

#[derive(Default)]
pub struct ApiKey(Arc<Mutex<Option<String>>>);
#[derive(Default)]
pub struct XmlPath(Mutex<Option<PathBuf>>);

pub struct EventSender {
    pub app_handle: AppHandle<Wry>,
    pub api_key: Arc<Mutex<Option<String>>>,
}
impl DebounceEventHandler for EventSender {
    fn handle_event(&mut self, res: DebounceEventResult) {
        let api_key = self.api_key.lock().unwrap().clone().unwrap_or_default();
        match res {
            Ok(events) => {
                for event in events.iter()
                    .filter(|e|e.path.extension() == Some(OsStr::new("xml")))
                    .filter(|e| e.kind == DebouncedEventKind::Any)
                {
                    let file_type = get_file_type(&event.path);
                    let res = match &file_type {
                        Some(FileType::StartList) => upload_file(&event.path, &api_key, "/start-lists"),
                        Some(FileType::ResultList) => upload_file(&event.path, &api_key, "/results"),
                        None => Err("Unrecognized file type".to_string())
                    };
                    match res {
                        Ok(_) => {
                            self.app_handle.emit_all("event-log", LogEntry {
                                severity: Success,
                                event: format!("{:?} uploaded", file_type.unwrap()),
                                filename: event.path.display().to_string(),
                            }).unwrap();
                        },
                        Err(e) => {
                            self.app_handle.emit_all("event-log", LogEntry {
                                severity: Error,
                                event: e,
                                filename: event.path.display().to_string(),
                            }).unwrap();
                        }
                    }
                }
            },
            Err(errors) => {
                for error in errors.iter()
                    .filter(|e|e.paths.iter().any(|p| p.extension() == Some(OsStr::new("xml"))))
                {
                    self.app_handle.emit_all("event-log", LogEntry {
                        severity: Error,
                        event: format!("{:?}", error),
                        filename: format!("{:?}", error.paths),
                    }).unwrap();
                }
            }
        }
    }
}

fn main() {
    tauri::Builder::default()
        .manage(XmlPath::default())
        .setup(|app| {
            let api_key = ApiKey::default();
            let event_sender = EventSender { app_handle: app.handle().clone(), api_key: api_key.0.clone() };
            app.manage(api_key);
            app.manage(ChangesWatcher::new(event_sender));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![pick_folder, set_api_key, watch, unwatch])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
