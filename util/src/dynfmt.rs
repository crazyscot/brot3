// (c) 2025 Ross Younger

fn strip_trailers(s: &str) -> &str {
    // Strip trailing zeroes (after the point)
    // Strip trailing decimal point, if it's not followed by any digits
    s.trim_end_matches('0').trim_end_matches('.')
}

/// Dynamic string formatting for floating point types.
///
/// This function attempts to mirror the behaviour of printf %g:
/// * formats in exponential (`e`) style if the exponent is less
///   than -4 or greater than the required precision;
/// * otherwise formats in normal (`f`) style;
/// * removes any trailing zeroes;
/// * removes any trailing decimal point.
///
/// # Example
/// ```
/// use util::dynamic_format;
/// let v = 1.234_567_8;
/// assert_eq!(dynamic_format(v, 6), "1.234568");
/// assert_eq!(dynamic_format(v, 0), "1.2");
/// assert_eq!(dynamic_format(0.000_001, 6), "1e-6");
/// ```
pub fn dynamic_format<V>(val: V, precision: usize) -> String
where
    f64: std::convert::From<V>,
{
    let val = f64::from(val);
    let precision = if precision == 0 { 1 } else { precision };
    let exponent = if val == 0. { 0. } else { val.abs().log10() };
    #[allow(clippy::cast_precision_loss)]
    let e_mode = exponent < -4.0 || exponent > precision as f64;
    let s = if e_mode {
        // I really want split_once_inclusive() but that's not available right now.
        let s = format!("{val:.precision$e}");
        if let Some((head, tail)) = s.split_once('e') {
            let head = strip_trailers(head);
            // Glue it back together, and we're done
            return format!("{head}e{tail}");
        }
        // this line shouldn't ever be reached but it's necessary to gracefully handle all outputs
        s
    } else {
        format!("{val:.precision$}")
    };
    strip_trailers(&s).to_owned()
}

/// Dynamic string formatting for floating point types.
///
/// This is a convenience macro with an optional `precision` parameter.
///
/// # Examples
/// ```
/// use util::dynfmt;
/// let v = 1.234_567_8;
/// assert_eq!(dynfmt!(v), "1.234568");
/// assert_eq!(dynfmt!(v, 0), "1.2");
/// ```
#[macro_export]
macro_rules! dynfmt {
    ($val:expr) => {
        $crate::dynamic_format($val, 6)
    };
    ($val: expr, $prec: expr) => {
        $crate::dynamic_format($val, $prec)
    };
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use core::f32;

    use crate::dynfmt;
    use pretty_assertions::assert_eq;

    // testcase helper macro: run both positive and negative versions of input data
    macro_rules! tc {
        ($val:expr, $expect: expr) => {
            assert_eq!(dynfmt!($val), $expect);
            {
                let mut tmp = String::from("-");
                tmp.push_str(&$expect);
                assert_eq!(dynfmt!(-$val), tmp);
            }
        };
        ($val:expr, $prec: expr, $expect: expr) => {
            assert_eq!(dynfmt!($val, $prec), $expect);
            {
                let mut tmp = String::from("-");
                tmp.push_str(&$expect);
                assert_eq!(dynfmt!(-$val, $prec), tmp);
            }
        };
    }

    #[test]
    fn zero() {
        assert_eq!(dynfmt!(0), "0");
    }

    #[test]
    fn float_mode() {
        let v = 1.234_567_8;
        // precision 0 is treated as 1
        tc!(v, 0, "1.2");
        // default precision is 6
        tc!(v, "1.234568");

        tc!(v, 1, "1.2");
        tc!(v, 2, "1.23");
        tc!(v, 3, "1.235");
        tc!(v, 4, "1.2346");
    }

    #[test]
    fn large_exp() {
        // default precision 6 => a large number will go to exponential notation
        let v = 1_000_000_000;
        tc!(v, "1e9");

        // specified precision is the cut-off
        let v = 1000;
        tc!(v, "1000");
        tc!(v, 3, "1000");
        tc!(v, 2, "1e3");

        // do we round sanely?
        let v = 888_888_888;
        tc!(v, "8.888889e8");
        tc!(v, 3, "8.889e8");
        let v = 999_999_999;
        tc!(v, "1e9");
        tc!(v, 3, "1e9");
    }
    #[test]
    fn small_exp() {
        // here, the cut-off is always -4
        tc!(0.001, "0.001");
        tc!(0.000_1, "0.0001");
        tc!(0.000_01, "1e-5");

        // more rounding checks
        let v = 0.000_000_001_234_567_89;
        tc!(v, "1.234568e-9");
        tc!(v, 2, "1.23e-9");
        tc!(v, 3, "1.235e-9");
        tc!(v, 4, "1.2346e-9");
    }
    #[test]
    fn subnormal() {
        assert_eq!(dynfmt!(f32::INFINITY), "inf");
        assert_eq!(dynfmt!(f32::NAN), "NaN");
    }
}
