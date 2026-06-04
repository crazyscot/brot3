#![allow(missing_docs)]

use std::{fs, path::PathBuf};

use brot3_lib::ui::UiState;

/// Ensure every .json file in the repository data/ directory can be loaded by `UiState::load_json`
#[test]
fn validate_data_json_files() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let data_dir = manifest.join("../data");

    let mut failed = Vec::new();

    let entries = match fs::read_dir(&data_dir) {
        Ok(e) => e,
        Err(e) => panic!(
            "Failed to read data directory {}: {}",
            data_dir.display(),
            e
        ),
    };

    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.eq_ignore_ascii_case("json"))
                {
                    match UiState::load_json(&path) {
                        Ok(_) => {}
                        Err(e) => {
                            failed.push((path.display().to_string(), format!("{e:?}")));
                        }
                    }
                }
            }
            Err(e) => failed.push((
                format!("<read_dir error>: {e}"),
                "read_dir error".to_string(),
            )),
        }
    }

    if !failed.is_empty() {
        eprintln!("Found {} invalid JSON data files:", failed.len());
        for (p, e) in &failed {
            eprintln!(" - {p}: {e}");
        }
        panic!("{} JSON data files failed to load", failed.len());
    }
}
