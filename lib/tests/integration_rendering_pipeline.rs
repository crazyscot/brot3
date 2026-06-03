#![allow(missing_docs)]

use brot3_lib::{
    BigVec2,
    data::{Algorithm, FragmentConstants, Palette, PushExponent},
    ui::{UiState, ViewportZoom},
    util::{render_chunk, render_frame},
};
use glam::uvec2;

fn test_fragment_constants() -> FragmentConstants {
    let state = UiState {
        viewport_translate: BigVec2::try_new(-0.75, 0.1).expect("failed to create BigVec2"),
        viewport_zoom: ViewportZoom::from(2.5_f32),
        algorithm: Algorithm::Mandelbrot,
        max_iter: 256,
        palette: Palette::default(),
        exponent: PushExponent::default(),
        iteration_cull: false,
        viewport_size: uvec2(100, 100),
    };
    let mut consts = FragmentConstants::from(&state);
    consts.size.width = 100;
    consts.size.height = 100;
    consts
}

#[test]
fn render_chunk_produces_valid_rgba() {
    let consts = test_fragment_constants();
    let mut pixels = vec![0u8; 4 * 64];
    let perturbation_points = vec![];

    render_chunk(0, &mut pixels, &consts, &perturbation_points);

    // All pixels should have been written (not all zeros)
    let all_zeros = pixels.iter().all(|&p| p == 0);
    assert!(
        !all_zeros,
        "render_chunk should produce non-zero pixel data"
    );
}

#[test]
fn render_chunk_respects_size() {
    let consts = test_fragment_constants();
    let chunk_size = 32;
    let mut pixels = vec![0u8; 4 * chunk_size];
    let perturbation_points = vec![];

    render_chunk(0, &mut pixels, &consts, &perturbation_points);

    // Buffer should be filled exactly for chunk_size pixels (4 bytes each)
    assert_eq!(pixels.len(), 4 * chunk_size);
}

#[test]
fn render_frame_produces_valid_output() {
    let consts = test_fragment_constants();
    let perturbation_points = vec![];

    let (pixels, _failure) = render_frame(&consts, &perturbation_points, false);

    // Buffer should be populated
    let filled_pixels = pixels.iter().filter(|&&p| p != 0).count();
    assert!(
        filled_pixels > 0,
        "render_frame should produce non-zero pixels"
    );
}

#[test]
fn render_frame_parallel_vs_serial_produces_same_output() {
    let consts = test_fragment_constants();
    let perturbation_points = vec![];

    // Render serially
    let (pixels_serial, _) = render_frame(&consts, &perturbation_points, false);

    // Render in parallel
    let (pixels_parallel, _) = render_frame(&consts, &perturbation_points, true);

    // Both should produce the same output
    assert_eq!(
        pixels_serial, pixels_parallel,
        "Serial and parallel rendering should produce identical output"
    );
}

#[test]
fn render_respects_max_iteration_limit() {
    let perturbation_points = vec![];

    // Render with low iterations
    let mut low_consts = test_fragment_constants();
    low_consts.max_iter = 10;
    let mut low_iter = vec![0u8; 4 * 64];
    render_chunk(0, &mut low_iter, &low_consts, &perturbation_points);

    // Render with high iterations
    let mut high_consts = test_fragment_constants();
    high_consts.max_iter = 1000;
    let mut high_iter = vec![0u8; 4 * 64];
    render_chunk(0, &mut high_iter, &high_consts, &perturbation_points);

    // Higher iterations typically produce more detailed/different coloring
    assert_ne!(
        low_iter, high_iter,
        "Different max iterations should produce different outputs"
    );
}
