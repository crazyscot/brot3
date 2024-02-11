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
    let mut raw_image = BytesMut::with_capacity(spec.width * spec.height * 4);
    // Temp gradient for testing:
    // RED goes from 0 to 255 over X
    // GREEN goes from 0 to 255 over Y
    // so we need to compute the tile offset into the whole image

    // spec.level is the log2 of the image dimension at this zoom level
    if spec.level > 63 {
        anyhow::bail!("level is too big");
    }
    let image_dimension: u64 = 1 << spec.level;
    let tile_count: u64 = image_dimension / spec.width as u64;
    let tile_step: f32 = 256.0 / tile_count as f32;
    let red_start: f32 = (tile_step * spec.dx as f32).floor();
    //let red_end: f32 = (tile_step * (spec.dx + 1) as f32).floor();
    let grn_start: f32 = (tile_step * spec.dy as f32).floor();
    //let grn_end: f32 = (tile_step * (spec.dy + 1) as f32).floor();
    let pixel_step: f32 = tile_step / spec.width as f32;
    //println!("level {} dx {} dy {} tc {tile_count} ts {tile_step} red {red_start}..{red_end} grn {grn_start}..{grn_end} pixstep {pixel_step}", spec.level, spec.dx, spec.dy);

    let mut red: f32;
    let mut grn: f32 = grn_start;
    for _ in 0..spec.height {
        red = red_start;
        for _ in 0..spec.width {
            raw_image.put_u8(red.floor() as u8);
            raw_image.put_u8(grn.floor() as u8);
            raw_image.put_u8(0);
            raw_image.put_u8(255);
            red += pixel_step;
        }
        grn += pixel_step;
    }

    Ok(RGBABlob {
        blob: Bytes::from(raw_image),
    })
}
