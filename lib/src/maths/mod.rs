//! Mathematical operations

mod float;
#[cfg(not(spirv))]
pub(crate) use float::FloatIsNear;

mod exponentiation;
pub use exponentiation::{Exponentiator, Power2, Power3, Power4, Power5, Power6, RealPower};
