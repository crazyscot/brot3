//! Save Image support
// (c) 2026 Ross Younger

use std::time::Instant;

use glam::{Vec2, Vec4, uvec2};
use shader::{
    data::PointResult,
    push_constants::{Flags, FragmentConstants},
};
use util::dprintln;

const DEBUG_SAVE: bool = false;

pub(crate) fn do_save_image(
    path: &std::path::Path,
    mut constants: FragmentConstants,
    perturbation_points: &[Vec2],
) -> anyhow::Result<()> {
    constants.flags |= Flags::NEEDS_REITERATE;
    constants.buffer_size = uvec2(0, 0).into();
    dprintln!(
        DEBUG_SAVE,
        "Saving image to {} with constants: {constants:?}",
        path.display()
    );

    let start = Instant::now();

    let mut lines = vec![Vec::new(); constants.size.height as usize];

    for (y, target) in lines.iter_mut().enumerate() {
        let mut tmp = render_line(&constants, y, perturbation_points);
        *target = std::mem::take(&mut tmp);
    }
    let flattened = lines.into_iter().flatten().collect::<Vec<_>>();

    let duration = start.elapsed();
    dprintln!(DEBUG_SAVE, "Rendered image in {duration:?}");

    let pngstart = Instant::now();
    let mut encoder = png::Encoder::new(
        std::fs::File::create(path)?,
        constants.size.width,
        constants.size.height,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.add_text_chunk("software".to_string(), "brot3".to_string())?;
    encoder.add_text_chunk("comment".to_string(), constants.display_string())?;
    // TODO, someday: get fragment constants to convert itself to/fro JSON, include that here.
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&flattened)?;
    dprintln!(DEBUG_SAVE, "Converted to PNG in {:?}", pngstart.elapsed());
    // TODO parallelise.
    // Will need to refactor perturbation buffer so we have a copy here. Perhaps it needs to be an
    // Arc or a Cow; could get awkward if we're working with it but the main loop wants to
    // update.
    Ok(())
}

/// Renders a single line of the image, returning the pixel data as RGBA values in pixel order.
fn render_line(constants: &FragmentConstants, y: usize, perturbation_points: &[Vec2]) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(constants.size.width as usize * 4); // RGBA
    let mut grid = [PointResult::default()];
    let mut pixel = Vec4::default();

    for x in 0..constants.size.width {
        #[allow(clippy::cast_precision_loss)]
        let frag_coord = Vec4::new(x as f32, y as f32, 0., 0.);
        shader::main_fs(
            frag_coord,
            constants,
            &mut grid,
            perturbation_points,
            &mut pixel,
        );
        pixel.w = 1.; // 100% alpha
        let bytes = (pixel * 255.0).as_u8vec4().to_array();
        // No endian issues here, at least on x86_64.
        pixels.extend_from_slice(&bytes);
    }
    pixels
}
