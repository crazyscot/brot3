#![allow(missing_docs)]

use bytemuck::{Pod, Zeroable};
use my_wgsl::WgslCompatible;

#[derive(Copy, Clone, Debug, Default, Zeroable, Pod, PartialEq, WgslCompatible)]
#[repr(C)]
#[allow(missing_docs)]
pub struct WgslFlags {
    pub bits: u32,
}

bitflags::bitflags! {
#[derive(Copy, Clone, Debug, Default, Zeroable, Pod, PartialEq)]
#[repr(transparent)]
#[allow(missing_docs)]
/// Flag bits for shader operation, packed into a u32 in the push constants
pub struct Flags : u32 {
    // TODO: Auto-sync these flag definitions with common.wgsl
    const NEEDS_REITERATE = 1 << 0;
    const INSPECTOR_ACTIVE = 1 << 1;
    const PERTURBATION_MODE = 1 << 2;
    const ITERATION_CULL = 1<<3;
    /// Set if we want to calculate distance estimation information (which is moderately expensive)
    const DISTANCE_ESTIMATE = 1<<4;

    const _ = !0;
}
}

impl From<Flags> for WgslFlags {
    fn from(flags: Flags) -> Self {
        Self { bits: flags.bits() }
    }
}

impl From<WgslFlags> for Flags {
    fn from(wgsl_flags: WgslFlags) -> Self {
        Self::from_bits_truncate(wgsl_flags.bits)
    }
}

impl Flags {
    /// Conditionally returns a flag value
    #[must_use]
    pub fn flag_if(condition: bool, flag: Flags) -> Flags {
        if condition { flag } else { Flags::empty() }
    }
}

impl WgslFlags {
    #[must_use]
    /// Wrapper for `Flags::contains`
    pub fn contains(&self, flag: Flags) -> bool {
        Flags::from(*self).contains(flag)
    }

    /// Wrapper for `Flags::set`
    pub fn set(&mut self, flag: Flags, value: bool) {
        let mut flags = Flags::from(*self);
        flags.set(flag, value);
        *self = flags.into();
    }
}
