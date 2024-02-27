// Tauri shared state: outstanding jobs list
// (c) 2024 Ross Younger

use rustc_hash::FxHashMap;
use tauri::async_runtime::RwLock;

pub struct Job {
    pub handle: tauri::async_runtime::JoinHandle<()>,
}

/// Outstanding jobs list, keyed by serial number. Provides thread-safe interior mutability.
#[derive(Default)]
pub struct OutstandingJobs {
    // Caution: This is thread-shared, needs interior mutability.
    map: RwLock<FxHashMap<u64, Job>>,
}

impl OutstandingJobs {
    pub async fn add(
        self: &OutstandingJobs,
        serial: u64,
        handle: tauri::async_runtime::JoinHandle<()>,
    ) {
        (*self.map.write().await).insert(serial, Job { handle });
    }

    // this is NOT marked as must_use, because it is legitimate to ignore the returned value.
    pub async fn remove_and_return(self: &OutstandingJobs, serial: u64) -> Option<Job> {
        (*self.map.write().await).remove(&serial)
    }
}
