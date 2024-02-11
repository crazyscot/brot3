// Viewer tile to PNG mapping
// (c) 2024 Ross Younger

use super::ViewerTileSpec;

use bytes::{BufMut, Bytes, BytesMut};
use serde::Serialize;

#[derive(Serialize)]
pub struct RGBABlob {
    blob: bytes::Bytes,
}

pub fn dummy_render(spec: ViewerTileSpec) -> anyhow::Result<RGBABlob> {
    // Holds a row of raw data
    let mut raw_image = BytesMut::with_capacity(spec.width * spec.height * 4);
    for _ in 0..raw_image.capacity() / 4 {
        // blue RGBA
        raw_image.put_u8(0);
        raw_image.put_u8(0);
        raw_image.put_u8(255);
        raw_image.put_u8(255);
    }

    Ok(RGBABlob {
        blob: Bytes::from(raw_image),
    })
}
