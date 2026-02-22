#![allow(dead_code)]

//! Configuration types for running SNAPHU.

pub mod defaults;

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
}
