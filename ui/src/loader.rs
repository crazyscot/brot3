// Tile loading logic, parallelisation and synchronisation
// (c) 2024 Ross Younger

use crate::engine;
use crate::types::TileCoordinate;

use core::future::Future;
use core::pin::Pin;

use slint::{Rgba8Pixel, SharedPixelBuffer};

pub(crate) struct LoadingTile {
    pub image: Pin<Box<dyn Future<Output = slint::Image>>>,
}

impl LoadingTile {
    pub(crate) fn new(coord: TileCoordinate) -> LoadingTile {
        LoadingTile {
            image: Box::pin(async move {
                let (image_send, image_recv) = tokio::sync::oneshot::channel();
                rayon::spawn(move || {
                    // If the LoadingTile is dropped, the channel becomes closed.
                    if !image_send.is_closed() {
                        let _a = image_send.send(engine::draw_tile(&coord));
                    }
                });
                let buffer = image_recv.await.unwrap().unwrap_or_else(|e| {
                    eprintln!("error drawing tile: {e}");
                    SharedPixelBuffer::<Rgba8Pixel>::new(1, 1)
                });
                slint::Image::from_rgba8(buffer)
            }),
        }
    }
}
