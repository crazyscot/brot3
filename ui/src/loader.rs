// Tile loading logic, parallelisation and synchronisation
// (c) 2024 Ross Younger

use crate::engine::{self};
use crate::types::{FinishedTile, PlottedTile, TileCoordinate};

use core::future::Future;
use core::pin::Pin;
use std::cell::RefCell;

use slint::{Rgba8Pixel, SharedPixelBuffer};

pub(crate) struct LoadingTile {
    pub data: Pin<Box<dyn Future<Output = PlottedTile>>>,
}

impl LoadingTile {
    pub(crate) fn new(coord: TileCoordinate, seed: Option<&FinishedTile>) -> LoadingTile {
        let seed_tile = seed.map(|v| v.tile.tile.clone());

        LoadingTile {
            data: Box::pin(async move {
                let (image_send, image_recv) = tokio::sync::oneshot::channel();
                rayon::spawn(move || {
                    // If the LoadingTile is dropped, the channel becomes closed.
                    if !image_send.is_closed() {
                        let _a = image_send.send(engine::draw_tile(&coord, seed_tile));
                    }
                });
                image_recv.await.unwrap().unwrap_or_else(|e| {
                    eprintln!("error drawing tile: {e}");
                    PlottedTile {
                        tile: RefCell::default(),
                        image: SharedPixelBuffer::<Rgba8Pixel>::new(1, 1),
                    }
                })
            }),
        }
    }
}
