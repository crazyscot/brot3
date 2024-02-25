// Tauri shared state: outstanding jobs list
// (c) 2024 Ross Younger

use rustc_hash::FxHashMap;
use tauri::async_runtime::RwLock;

/// Outstanding jobs list, keyed by serial number. Provides thread-safe interior mutability.
#[derive(Default)]
pub struct OutstandingJobs {
    // Caution: This is thread-shared, needs interior mutability.
    map: RwLock<FxHashMap<u64, tauri::async_runtime::JoinHandle<()>>>,
}

impl OutstandingJobs {
    pub fn add(self: &OutstandingJobs, serial: u64, handle: tauri::async_runtime::JoinHandle<()>) {
        (*self.map.blocking_write()).insert(serial, handle);
    }
    pub fn remove_and_return(
        self: &OutstandingJobs,
        serial: u64,
    ) -> Option<tauri::async_runtime::JoinHandle<()>> {
        (*self.map.blocking_write()).remove(&serial)
    }
}