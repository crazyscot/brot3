use std::{borrow::Cow, fmt::Display, str::FromStr};

use brot3_idl::Colourer;
use internment::Intern;
use num_traits::AsPrimitive as _;

use crate::ui::{ShaderVariant, UiState};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// A key used to identify a specific shader variant and colourer combination.
pub struct ShaderKey {
    /// The shader variant.
    pub variant: ShaderVariant,
    /// The colourer used.
    pub colourer: Colourer,
}

impl ShaderKey {
    /// Returns an interned string representation of the shader key.
    #[must_use]
    pub fn interned(&self) -> Cow<'static, str> {
        let ttt = Intern::new(self.to_string());
        Cow::Borrowed(ttt.as_ref())
    }
}

impl From<ShaderKey> for usize {
    fn from(key: ShaderKey) -> Self {
        (key.variant as usize) | (key.colourer as usize) << 8
    }
}

impl From<usize> for ShaderKey {
    fn from(value: usize) -> Self {
        // This must match From<ShaderKey> for usize
        let tt = (value >> 8) & 0xff;
        Self {
            variant: ShaderVariant::from_repr(value & 0xff).unwrap_or_default(),
            colourer: Colourer::from_repr(tt.as_()).unwrap_or_default(),
        }
    }
}

impl From<&UiState> for ShaderKey {
    fn from(state: &UiState) -> Self {
        Self {
            variant: state.into(),
            colourer: *state.palette.colourer,
        }
    }
}

#[derive(thiserror::Error, Debug, derive_more::Display)]
#[allow(missing_copy_implementations)]
pub struct InvalidShaderKey;

impl FromStr for ShaderKey {
    type Err = InvalidShaderKey;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // split on '_'
        let parts: Vec<&str> = s.split('_').collect();
        Ok(Self {
            variant: parts
                .first()
                .and_then(|s| ShaderVariant::from_str(s).ok())
                .ok_or(InvalidShaderKey)?,
            colourer: parts
                .get(1)
                .and_then(|s| Colourer::from_str(s).ok())
                .ok_or(InvalidShaderKey)?,
        })
    }
}

impl Display for ShaderKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}_{:?}", self.variant, self.colourer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default() {
        let k = ShaderKey::default();
        let u: usize = k.into();
        let k2: ShaderKey = u.into();
        assert_eq!(k, k2);
        let s = format!("{k}");
        assert!(s.contains(&format!("{:?}", k.variant)));
        assert!(s.contains(&format!("{:?}", k.colourer)));
    }

    #[test]
    fn round_trip() {
        let k1 = ShaderKey {
            variant: ShaderVariant::MandelbrotPow2,
            colourer: Colourer::WhiteFade,
        };
        let k2 = ShaderKey {
            variant: ShaderVariant::MandelbrotPow2,
            colourer: Colourer::BlackFade,
        };
        let k3 = ShaderKey {
            variant: ShaderVariant::MandelbrotPow2,
            colourer: Colourer::OneLoneCoder,
        };
        assert_ne!(k1, k2);
        assert_ne!(k1, k3);
        for k in [k1, k2, k3] {
            let u: usize = k.into();
            let str = format!("{k}");
            let rev = ShaderKey::from(u);
            let rev2 = ShaderKey::from_str(&str).unwrap();
            println!(
                "ShaderKey: {k:?}; display {k}; int 0x{u:x}; from int {rev:?}; from display {rev2:?}"
            );
            assert_eq!(k, rev);
            assert_eq!(k, rev2);
        }
    }

    #[test]
    fn colourer_fromstr_regression() {
        use std::str::FromStr;
        let s1 = "WhiteFade";
        let s2 = "BlackFade";
        let c1 = Colourer::from_str(s1).unwrap();
        let c2 = Colourer::from_str(s2).unwrap();
        assert_eq!(c1, Colourer::WhiteFade);
        assert_eq!(c2, Colourer::BlackFade);
    }

    #[test]
    fn invalid_input() {
        assert!(
            ShaderKey::from_str("InvalidInput")
                .inspect(|d| println!("got: {d:?}"))
                .is_err()
        );
    }
}
