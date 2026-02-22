#![allow(dead_code)]

//! Configuration types for running SNAPHU.

pub mod defaults;

use core::ffi::c_long;

/// Placeholder for the eventual full configuration structure.
#[derive(Debug, Clone, Default)]
pub struct RunConfig {
    pub description: String,
}

/// Returns `true` if the string represents a truthy config value.
///
/// Recognises the same literals as the C `IsTrue()`: `TRUE`, `true`, `True`,
/// `1`, `y`, `Y`, `yes`, `YES`, `Yes`.
pub fn is_true(s: &str) -> bool {
    matches!(
        s,
        "TRUE" | "true" | "True" | "1" | "y" | "Y" | "yes" | "YES" | "Yes"
    )
}

/// Returns `true` if the string represents a falsy config value.
///
/// Recognises the same literals as the C `IsFalse()`: `FALSE`, `false`,
/// `False`, `0`, `n`, `N`, `no`, `NO`, `No`.
pub fn is_false(s: &str) -> bool {
    matches!(
        s,
        "FALSE" | "false" | "False" | "0" | "n" | "N" | "no" | "NO" | "No"
    )
}

/// Uses C-equivalent `strtod` behavior plus SNAPHU's error checks.
///
/// Returns `Some(value)` when the full string is consumed and the parsed value
/// is finite. Returns `None` when parsing fails, extra characters remain, or
/// the parsed value is infinite.
///
/// This intentionally preserves the C quirk that an empty string parses as `0`.
pub fn string_to_double(input: &str) -> Option<f64> {
    if input.is_empty() {
        return Some(0.0);
    }

    // `strtod` accepts leading whitespace. We preserve that behavior while
    // still rejecting any trailing characters.
    let normalized = input.trim_start();
    let value = normalized.parse::<f64>().ok()?;
    if value.is_infinite() {
        None
    } else {
        Some(value)
    }
}

/// Uses C-equivalent `strtol(..., base=10)` behavior plus SNAPHU's checks.
///
/// Returns `Some(value)` when the full string is consumed as base-10 and does
/// not hit the C sentinel checks. Returns `None` when parsing fails, extra
/// characters remain, or when value equals `LONG_MAX`/`LONG_MIN` (matching the
/// legacy function's overflow test).
///
/// This intentionally preserves the C quirk that an empty string parses as `0`.
pub fn string_to_long(input: &str) -> Option<c_long> {
    if input.is_empty() {
        return Some(0);
    }

    // `strtol` accepts leading whitespace. We preserve that behavior while
    // still rejecting any trailing characters.
    let normalized = input.trim_start();
    let value = normalized.parse::<c_long>().ok()?;
    if value == c_long::MAX || value == c_long::MIN {
        None
    } else {
        Some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_true_accepts_all_truthy_variants() {
        for s in &["TRUE", "true", "True", "1", "y", "Y", "yes", "YES", "Yes"] {
            assert!(is_true(s), "expected is_true({s:?}) == true");
        }
    }

    #[test]
    fn is_true_rejects_falsy_and_invalid() {
        for s in &["FALSE", "false", "0", "n", "no", "maybe", ""] {
            assert!(!is_true(s), "expected is_true({s:?}) == false");
        }
    }

    #[test]
    fn is_false_accepts_all_falsy_variants() {
        for s in &["FALSE", "false", "False", "0", "n", "N", "no", "NO", "No"] {
            assert!(is_false(s), "expected is_false({s:?}) == true");
        }
    }

    #[test]
    fn is_false_rejects_truthy_and_invalid() {
        for s in &["TRUE", "true", "1", "y", "yes", "maybe", ""] {
            assert!(!is_false(s), "expected is_false({s:?}) == false");
        }
    }

    #[test]
    fn string_to_double_parses_valid_inputs() {
        assert_eq!(string_to_double("1.25"), Some(1.25));
        assert_eq!(string_to_double("  -2.5"), Some(-2.5));
        assert_eq!(string_to_double(""), Some(0.0));
    }

    #[test]
    fn string_to_double_rejects_partial_and_infinite_values() {
        assert_eq!(string_to_double("1.5x"), None);
        assert_eq!(string_to_double("1.5 "), None);
        assert_eq!(string_to_double("abc"), None);
        assert_eq!(string_to_double("1e10000"), None);
    }

    #[test]
    fn string_to_double_accepts_nan_like_c_strtod() {
        let parsed = string_to_double("nan").expect("nan should parse");
        assert!(parsed.is_nan());
    }

    #[test]
    fn string_to_long_parses_valid_inputs() {
        assert_eq!(string_to_long("42"), Some(42));
        assert_eq!(string_to_long("  -17"), Some(-17));
        assert_eq!(string_to_long(""), Some(0));
    }

    #[test]
    fn string_to_long_rejects_partial_invalid_and_extreme_values() {
        assert_eq!(string_to_long("12x"), None);
        assert_eq!(string_to_long("12 "), None);
        assert_eq!(string_to_long("abc"), None);
        assert_eq!(string_to_long(&c_long::MAX.to_string()), None);
        assert_eq!(string_to_long(&c_long::MIN.to_string()), None);
    }
}
