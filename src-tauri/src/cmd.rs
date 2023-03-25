use tauri::api::dialog::blocking::FileDialogBuilder;
use crate::{ApiKey, changes_watcher::ChangesWatcher, XmlPath};

#[tauri::command]
pub async fn pick_folder(state: tauri::State<'_, XmlPath>) -> Result<String, ()> {
    if let Some(folder_path) = FileDialogBuilder::new().pick_folder() {
        let mut xml_path = state.0.lock().unwrap();
        *xml_path = Some(folder_path.clone());
        return Ok(folder_path.display().to_string());
    }
    Err(())
}

#[tauri::command]
pub fn set_api_key(new_api_key: String, state: tauri::State<'_, ApiKey>) {
    let mut api_key = state.0.lock().unwrap();
    *api_key = Some(new_api_key).filter(|s| !s.is_empty());
}

#[tauri::command]
pub fn watch(debouncer: tauri::State<'_, ChangesWatcher>, xml_path: tauri::State<'_, XmlPath>) -> Result<(), String> {
    let xml_path = xml_path.0.lock().unwrap().clone();
    match xml_path {
        Some(xml_path) => {
            debouncer.watch(xml_path)?;
            Ok(())
        }
        None => Err("No folder selected".to_string())
    }
}

#[tauri::command]
pub fn unwatch(debouncer: tauri::State<'_, ChangesWatcher>) -> Result<(), String> {
    debouncer.unwatch()
}