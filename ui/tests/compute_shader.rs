#![allow(missing_docs)]

use std::{path::Path, time::Instant};

use brot3_lib::{data::FragmentConstants, ui::UiState};
use brot3_ui::compute::ComputeController;
use glam::uvec2;

#[test]
#[ignore = "This is a performance test, not a correctness test."]
fn compute_shader_test() -> Result<(), Box<dyn std::error::Error>> {
    let n_passes = 101;
    let render_size = uvec2(3840, 2160);
    let mut times = Vec::new();
    let start = Instant::now();
    let mut controller = ComputeController::new(render_size, n_passes)?;
    times.push(("initialization", start.elapsed()));
    let consts = FragmentConstants {
        size: render_size.into(),
        ..Default::default()
    };
    let mut frame_data = Vec::with_capacity(render_size.element_product() as usize);

    let start = Instant::now();
    let raw_times = controller.run(
        consts,
        render_size.extend(1),
        n_passes,
        &[], // no perturbation points for this test
        |rgba| {
            frame_data.clear();
            frame_data.extend_from_slice(rgba);
        },
    )?;
    let compute_time = start.elapsed();
    times.push(("compute", compute_time));
    times.push(("  compute per-pass", compute_time / n_passes));
    // N.B. Wallclock time is often much higher than GPU execution time.
    // Tune the workload & dispatch size to suit.

    let start = Instant::now();
    let state = UiState::try_from(&consts).unwrap();
    brot3_ui::write_png(Path::new("compute_test_output.png"), &state, &frame_data)?;
    times.push(("save PNG", start.elapsed()));

    println!("CPU Timing (wallclock):");
    for (name, time) in times {
        println!("  {name}: {time:?}");
    }
    if raw_times.is_empty() {
        println!("GPU Timing: No timestamps collected");
    } else {
        process_times(n_passes, &raw_times, true);
    }

    Ok(())
}

fn process_times(n_passes: u32, raw_times: &[u64], print_times: bool) {
    use easy_cast::traits::Conv as _;
    println!("GPU Timing (kernel execution time):");
    /*
    println!("Timing results (nanoseconds):");
    for (i, timestamp) in raw_times.iter().enumerate() {
        if *timestamp != 0 {
            println!("  Query {i:3}: {timestamp}");
        }
    }
    */
    let mut pass_times = Vec::with_capacity(n_passes as usize);
    let base_time = raw_times[0];
    raw_times
        .iter()
        .skip(1)
        .step_by(2)
        .zip(raw_times.iter().skip(2).step_by(2))
        .enumerate()
        .for_each(|(i, (a, b))| {
            let start_time = a.saturating_sub(base_time);
            let elapsed = b.saturating_sub(*a);
            assert!(elapsed > 0);
            // skip the first pass, which is often an outlier
            if i != 0 {
                if print_times {
                    println!("  Pass {i:3}: start {start_time:7}ns, elapsed {elapsed:7}ns");
                }
                pass_times.push(elapsed);
            }
        });
    println!(
        "  Total GPU execution time: {}ns",
        raw_times[raw_times.len() - 1].wrapping_sub(base_time)
    );
    let summarised_passes = n_passes - 1;
    let variance: average::Variance = pass_times.into_iter().map(f64::conv).collect();
    println!(
        "  Over {summarised_passes} passes: Average {:.2}ns; stddev {:.2}ns",
        variance.mean(),
        variance.sample_variance().sqrt()
    );
}
