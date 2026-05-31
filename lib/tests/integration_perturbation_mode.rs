#![allow(missing_docs)]

use brot3_lib::ui::ViewportZoom;

#[test]
fn perturbation_mode_required_at_high_zoom() {
    let zoom = ViewportZoom::from(1e10_f32);
    assert!(
        zoom.requires_perturbation_mode(),
        "Perturbation mode should be required at 1e10 zoom"
    );
}

#[test]
fn perturbation_mode_not_required_at_low_zoom() {
    let zoom = ViewportZoom::from(1.0_f32);
    assert!(
        !zoom.requires_perturbation_mode(),
        "Perturbation mode should not be required at 1.0 zoom"
    );
}

#[test]
fn perturbation_mode_threshold_boundary() {
    #![allow(clippy::cast_possible_truncation)]
    // Test boundary around the perturbation mode threshold (1.0e4)
    let zoom_values = vec![
        (1.0, false),
        (10.0, false),
        (100.0, false),
        (1000.0, false),
        (1e4 - 1.0, false),
        (1e4, false),      // Exactly at boundary
        (1e4 + 1.0, true), // Just above boundary
        (1e10, true),
    ];

    for (zoom, should_require_perturbation) in zoom_values {
        let vz = ViewportZoom::from(zoom as f32);
        let requires = vz.requires_perturbation_mode();

        assert_eq!(
            requires, should_require_perturbation,
            "Zoom {zoom} should require perturbation={should_require_perturbation}",
        );
    }
}

#[test]
fn viewport_zoom_clamp_respects_perturbation_mode() {
    let high_zoom = ViewportZoom::from(1e10_f32);

    // Without perturbation mode, should clamp to MAX_ZOOM_STANDARD
    let clamped_standard = high_zoom.clamp_to_mode(false);
    assert!(
        !clamped_standard.requires_perturbation_mode(),
        "Clamped zoom without perturbation mode should not require perturbation"
    );

    // With perturbation mode, should keep high zoom
    let clamped_perturb = high_zoom.clamp_to_mode(true);
    assert!(
        clamped_perturb.requires_perturbation_mode(),
        "Clamped zoom with perturbation mode should require perturbation"
    );
}
