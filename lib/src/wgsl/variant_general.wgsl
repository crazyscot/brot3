// variant: general

const ESCAPE_THRESHOLD_SQ: f32 = 100.0;
const ESCAPE_THRESHOLD_LOGLOG2: f32 = log2(log2(ESCAPE_THRESHOLD_SQ)); // 1.732_020_9;

fn mandelbrot_pixel(coord: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
    let pixel_spacing = 1.0 / max(1.0, constants.viewport_zoom * max(1.0, size.y));
    let complex_offset = (coord - 0.5 * size) * pixel_spacing;
    let c = complex_offset + constants.viewport_translate;

    var z = vec2<f32>(0.0, 0.0);
    var iter: u32 = 0u;
    let max_iter = max(1u, constants.max_iter);
    var norm_sqr: f32;

    // TODO: pow=2 & standard Mandelbrot only
    {
        // SHORT CUT: Points within the cardioid or period-2 bulb.
        let cr14 = c.x - 0.25;
        let im2 = c.y * c.y;
        let q = cr14 * cr14 + im2;
        let cr1 = c.x + 1.0;
        if (q * (q+cr14) <= 0.25 * im2) | (cr1 * cr1 + im2 < 0.0625) {
            return vec3<f32>(0.0, 0.0, 0.0);
        }
    }

    loop {
        // TODO: pow2 vs others (some careful fragmenting, I think)
        z = vec2<f32>(
            z.x * z.x - z.y * z.y + c.x,
            2.0 * z.x * z.y + c.y,
        );
        iter = iter + 1u;
        norm_sqr = dot(z, z);
        if (iter >= max_iter) | (norm_sqr > ESCAPE_THRESHOLD_SQ) {
            break;
        }
    }

    let escaped = norm_sqr >= ESCAPE_THRESHOLD_SQ;
    if (!escaped) { // This early return seems to not affect performance noticeably.
        return vec3<f32>(0.0, 0.0, 0.0);
    }

    // Fractional escape count: See http://linas.org/art-gallery/escape/escape.html
    // The log(exponent) term is necessary for powers other than 2.
    // Note that the log of the exponent is not allowed to be 0 or subnormal (we divide by
    // it below), so we special case those regions.

    // For points that escape immediately, the log-log term is negative, and the escape
    // count is very close to 1.0. This can cause precision issues and even NaNs when we
    // divide by the exponent log2 below.
    // If we move to a branchless style, clamp norm_sqr to 1.000001 here.

    // take two logs, avoiding NaN.
    // by the logarithm of a power law,
    // z.norm().log() === z.norm_sqr().log() * 0.5
    let log_term = log2(log2(norm_sqr) * 0.5);
    // TODO: pow2: log_term /= log2(power); // at 2.0, this is a no-op

    let iter_fraction = 1.0 + ESCAPE_THRESHOLD_LOGLOG2 - log_term;

    if ((constants.flags.bits & FLAG_ITERATION_CULL) != 0u) {
        iter = cull_iterations(iter);
    }
    var result = PointResult(iter, iter_fraction, 0.0, 0.0, BoundaryClass_Indeterminate);

    return colour_pixel(result);
}
