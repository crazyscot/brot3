// Processing side of the Save Image workflow
// (c) 2024 Ross Younger

use anyhow::Context;
use brot3_engine::{
    colouring,
    fractal::{Tile, TileSpec},
    render::{self, autodetect_extension, Renderer},
};
use rayon::prelude::*; // par_iter_mut
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};
use tauri::Manager;

use crate::{mutable_util::InnerMutable, render_spec::RenderSpec, util::GenericError};

#[derive(Clone)]
pub(crate) struct SaveStateInner {
    dir: OsString,
}

impl SaveStateInner {
    /// Default save directory (OS-specific)
    pub fn default_save_dir() -> PathBuf {
        tauri::api::path::picture_dir()
            .or_else(tauri::api::path::home_dir)
            .unwrap_or_default()
    }
}

impl Default for SaveStateInner {
    fn default() -> Self {
        let path = Self::default_save_dir();
        Self {
            dir: path.as_os_str().into(),
        }
    }
}

pub type SaveState = InnerMutable<SaveStateInner>;

#[tauri::command]
pub async fn save_image_workflow(spec: RenderSpec, app_handle: tauri::AppHandle) {
    let _ = save_image_workflow_inner(spec, app_handle.clone())
        .await
        .map_err(|e| {
            println!("Error in save_image_workflow: {e}");
            let _ = app_handle.emit_all(
                "genericError",
                GenericError {
                    error: e.to_string(),
                },
            );
        });
}
async fn save_image_workflow_inner(
    spec: RenderSpec,
    app_handle: tauri::AppHandle,
) -> anyhow::Result<()> {
    use tauri::api::dialog::FileDialogBuilder;

    let col_selection =
        colouring::Selection::from_str(&spec.colourer).context("colourer selection")?;
    let colourer = colouring::factory(col_selection);

    let tile_spec: TileSpec = RenderSpec::try_into(spec.clone())?;

    let default_dir = app_handle.state::<SaveState>().clone_async().await.dir;
    let file_name = tile_spec.to_string() + ".png";

    FileDialogBuilder::new()
        .set_title("Save fractal image (PNG)...")
        .set_file_name(&file_name)
        .set_directory(default_dir.as_os_str().to_str().unwrap_or(""))
        .save_file(move |file_path| {
            // the file path is `None` if the user closed the dialog
            if let Some(mut path) = file_path {
                do_save(&tile_spec, colourer, &path, app_handle.clone());
                path.pop();
                let new_state = SaveStateInner {
                    dir: path.as_os_str().to_os_string(),
                };
                app_handle.state::<SaveState>().replace_blocking(&new_state);
            }
        });
    Ok(())
}

fn do_save(
    spec: &TileSpec,
    colourer: colouring::Instance,
    path: &Path,
    app_handle: tauri::AppHandle,
) {
    let _ = do_save_inner(spec, colourer, path).map_err(|e| {
        println!("Error saving: {e}");
        let _ = app_handle.emit_all(
            "genericError",
            GenericError {
                error: e.to_string(),
            },
        );
    });
}

fn do_save_inner(
    spec: &TileSpec,
    colourer: colouring::Instance,
    path: &Path,
) -> anyhow::Result<()> {
    let filename_osstr = path.to_str().context("Filename conversion failed")?;
    let render_selection: render::Selection =
        *autodetect_extension(filename_osstr).context("Unknown file extension")?;
    let renderer = render::factory(render_selection);
    let splits = spec.split(5, 0)?;
    let mut tiles: Vec<Tile> = splits.iter().map(|ts| Tile::new(ts, 0)).collect();
    let time1 = SystemTime::now();
    tiles.par_iter_mut().for_each(|t| t.plot());
    // SOMEDAY: Consider progress reporting
    let time2 = SystemTime::now();
    let result = renderer.render_file(filename_osstr, spec, &tiles, colourer);
    let time3 = SystemTime::now();
    if false {
        println!(
            "plotted in {:?}, rendered in {:?}",
            time2.duration_since(time1).unwrap_or_default(),
            time3.duration_since(time2).unwrap_or_default(),
        )
    }

    result
}
