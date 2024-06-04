// Viewer tile to PNG mapping
// (c) 2024 Ross Younger

use super::ViewerTileSpec;
use crate::OutstandingJobs;
use brot3_engine::{
    colouring,
    fractal::{self, Algorithm, Point, Scalar, Tile, TileCache, TileSpec},
    render,
    util::listable::ListItem,
};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tauri::Manager;

// TileCache has interior mutability and only requires an immutable reference.
// TODO: Tune cache carefully! 244 seems sufficient to fill my 2k screen; 840ish for my 4k.
static TILE_CACHE: Lazy<TileCache> = Lazy::new(|| TileCache::new(1000));

#[derive(Serialize, Clone)]
pub struct TileResponse {
    serial: u64,
    rgba_blob: bytes::Bytes,
}

#[derive(Serialize, Clone)]
pub struct TileError {
    serial: u64,
    error: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializablePoint {
    re: Scalar,
    im: Scalar,
}
impl From<Point> for SerializablePoint {
    fn from(item: Point) -> Self {
        SerializablePoint {
            re: item.re,
            im: item.im,
        }
    }
}
impl From<SerializablePoint> for Point {
    fn from(item: SerializablePoint) -> Self {
        Point {
            re: item.re,
            im: item.im,
        }
    }
}

/// Description of a view into the fractal.
/// This could also be the overall fractal dimensions (which we refer to as its _metadata_).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FractalView {
    pub origin: SerializablePoint,
    pub axes_length: SerializablePoint,
}

fn draw_tile(
    spec: &ViewerTileSpec,
    app_handle: &tauri::AppHandle,
    cache: &TileCache,
) -> anyhow::Result<(), String> {
    let colourer_requested = &spec.colourer;
    let colourer = colouring::decode(colourer_requested).map_err(|e| e.to_string())?;
    let engine_spec = TileSpec::try_from(spec).map_err(|e| e.to_string())?;
    // Is it in cache?
    let mut tile = cache.get(&engine_spec);
    if tile.is_none() {
        let mut new_tile = Tile::new(&engine_spec, 0);
        new_tile.plot();
        cache.insert(new_tile); // Consumes new_tile
        tile = cache.get(&engine_spec); // ... and then retrieves a reference to what was new_tile
                                        // println!("miss {:?} len={}", engine_spec, cache.len());
    } else {
        // println!("hit {:?}", engine_spec);
    }
    let response = render::as_rgba_from_cache(tile, colourer)?;
    app_handle
        .emit_all(
            "tile_complete",
            TileResponse {
                serial: spec.serial,
                rgba_blob: response.into(),
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn start_tile(spec: ViewerTileSpec, app_handle: tauri::AppHandle) -> Result<(), String> {
    let app_handle_copy = app_handle.clone();
    let serial = spec.serial;

    let job = tauri::async_runtime::spawn(async move {
        let serial = spec.serial;
        if let Err(error) = draw_tile(&spec, &app_handle, &TILE_CACHE) {
            let _ = app_handle.emit_all("tile_error", TileError { serial, error });
        }
        app_handle
            .state::<OutstandingJobs>()
            .remove_and_return(serial)
            .await;
    });
    app_handle_copy
        .state::<OutstandingJobs>()
        .add(serial, job)
        .await;
    Ok(())
}

#[tauri::command]
pub async fn abort_tile(
    serial: u64,
    jobs: tauri::State<'_, OutstandingJobs>,
) -> Result<(), String> {
    if let Some(h) = jobs.remove_and_return(serial).await {
        h.handle.abort();
    }
    Ok(())
}

#[tauri::command]
/// Retrieve metadata for the given fractal
pub fn get_metadata(algorithm: String) -> anyhow::Result<FractalView, String> {
    let algorithm_instance = fractal::decode(&algorithm).map_err(|e| e.to_string())?;
    let origin = algorithm_instance.default_centre() - 0.5 * algorithm_instance.default_axes();
    Ok(FractalView {
        origin: origin.into(),
        axes_length: algorithm_instance.default_axes().into(),
    })
}

#[tauri::command]
/// List available fractal algorithms or colourers
pub fn list_items(what: String) -> anyhow::Result<Vec<ListItem>, String> {
    use brot3_engine::{fractal, util::listable};
    if what == "fractals" {
        Ok(listable::list_original_case::<fractal::Selection>().collect())
    } else if what == "colourers" {
        Ok(listable::list_original_case::<colouring::Selection>().collect())
    } else {
        Err(format!("Unhandled selection request {}", what))
    }
}
