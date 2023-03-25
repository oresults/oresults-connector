use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{Debouncer, new_debouncer};
use crate::EventSender;

pub struct ChangesWatcher {
    pub debouncer: Mutex<Debouncer<RecommendedWatcher>>,
    watched_path: Mutex<Option<PathBuf>>,
}

impl ChangesWatcher {
    pub fn new(event_sender: EventSender) -> Self {
        let debouncer = new_debouncer(Duration::from_secs(2), None, event_sender).unwrap();
        Self {
            debouncer: Mutex::new(debouncer),
            watched_path: Mutex::new(None)
        }
    }
    pub fn watch(&self, path: PathBuf) -> Result<(), String> {
        self.unwatch()?;
        let mut debouncer = self.debouncer.lock().unwrap();
        let mut watched_path = self.watched_path.lock().unwrap();
        debouncer.watcher().watch(&path, RecursiveMode::Recursive).map_err(|e| e.to_string())?;
        *watched_path = Some(path);
        Ok(())
    }

    pub fn unwatch(&self) -> Result<(), String> {
        let mut debouncer = self.debouncer.lock().unwrap();
        let mut watched_path = self.watched_path.lock().unwrap();
        if let Some(watched_path) = watched_path.as_ref() {
            debouncer.watcher().unwatch(watched_path).map_err(|e| e.to_string())?;
        }
        *watched_path = None;
        Ok(())
    }
}