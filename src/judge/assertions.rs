//! Assertion macros for LLM-as-a-judge testing.
//!
//! This module provides convenient macros for asserting judgment results in tests,
//! making it easy to write readable and maintainable test suites.

// The assertion macros work directly with the Judgment type

/// Assert that a judgment passes
#[macro_export]
macro_rules! assert_passes {
    ($judgment:expr) => {
        assert!($judgment.passes, "Judgment failed: {}", $judgment.reasoning);
    };
    ($judgment:expr, $($arg:tt)*) => {
        assert!($judgment.passes, $($arg)*);
    };
}

/// Assert that a judgment fails
#[macro_export]
macro_rules! assert_fails {
    ($judgment:expr) => {
        assert!(!$judgment.passes, "Expected failure but judgment passed: {}", $judgment.reasoning);
    };
    ($judgment:expr, $($arg:tt)*) => {
        assert!(!$judgment.passes, $($arg)*);
    };
}

/// Assert minimum confidence level
#[macro_export] 
macro_rules! assert_confidence {
    ($judgment:expr, $min:expr) => {
        if let Some(confidence) = $judgment.confidence {
            assert!(confidence >= $min, "Confidence {} below {}: {}", confidence, $min, $judgment.reasoning);
        }
    };
}