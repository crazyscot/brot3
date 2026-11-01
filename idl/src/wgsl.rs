//! WGSL definitions export

use std::sync::LazyLock;

#[allow(clippy::wildcard_imports)]
use crate::*;

macro_rules! wgsl_types {
    ($s:ident; $m:ident) => {
        $m!(
            $s;
            Size,
            WgslVec2,
            PushExponent,
            Palette,
            WgslFlags,
            FragmentConstants,
            PointResult,
        );
    };
}

macro_rules! wgsl_enums {
    ($s:ident; $m:ident) => {
        $m!(
            $s;
            Algorithm,
            BoundaryClass,
            ColourStyle,
            Colourer,
            Modifier,
            NumericType,
            RenderMode,
        );
    };
}

macro_rules! push_wgsl_defs {
    ($s:ident; $($ty:ty),+ $(,)?) => {
        $(
            $s.push_str(<$ty as my_wgsl::WgslCompatible>::WGSL_DEFINE);
            $s.push('\n');
        )+
    };
}

macro_rules! push_wgsl_enumdefs {
    ($s:ident; $($ty:ty),+ $(,)?) => {
        $(
            $s.push_str(<$ty>::WGSL_DEFINE);
            $s.push('\n');
        )+
    };
}

/// The complete collection of WGSL definitions exported by this module
pub static ALL: LazyLock<String> = LazyLock::new(|| {
    let mut s = String::new();
    wgsl_types!(s; push_wgsl_defs);
    wgsl_enums!(s; push_wgsl_enumdefs);
    s
});

#[cfg(test)]
mod tests {
    #[test]
    fn wgsl_defines() {
        println!("{}", *super::ALL);
        assert!(super::ALL.contains("struct FragmentConstants"));
        assert!(super::ALL.contains("struct Palette"));
        assert!(super::ALL.contains("struct PushExponent"));
        assert!(super::ALL.contains("Algorithm_Mandelbrot"));
        assert!(super::ALL.contains("BoundaryClass_VeryClose"));
        assert!(super::ALL.contains("ColourStyle_Continuous"));
        assert!(super::ALL.contains("Colourer_Mandy"));
        assert!(super::ALL.contains("Modifier_Filaments"));
        assert!(super::ALL.contains("NumericType_Integer"));
        assert!(super::ALL.contains("RenderMode_Gpu"));
    }
}
