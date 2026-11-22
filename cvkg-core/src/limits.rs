//! Explicit budgets for parsing untrusted bytes.
//!
//! Parsers check declared sizes and nesting before allocating or recursing.
//! A `Limits` value travels with the byte slice instead of hiding in a global.

use thiserror::Error;

/// A declared size exceeded its budget.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LimitError {
    #[error("input declares {declared} bytes, limit is {limit}")]
    TooManyBytes { declared: usize, limit: usize },
    #[error("input declares {declared} {what}, limit is {limit}")]
    TooManyItems {
        what: &'static str,
        declared: usize,
        limit: usize,
    },
    #[error("input nests deeper than {limit} levels")]
    TooDeep { limit: usize },
}

/// Budgets for one untrusted input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    pub max_bytes: usize,
    pub max_depth: usize,
    pub max_nodes: usize,
    pub max_attributes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bytes: 64 * 1024 * 1024,
            max_depth: 128,
            max_nodes: 1_000_000,
            max_attributes: 512,
        }
    }
}

impl Limits {
    pub fn checked_len(self, len: usize) -> Result<(), LimitError> {
        if len > self.max_bytes {
            Err(LimitError::TooManyBytes {
                declared: len,
                limit: self.max_bytes,
            })
        } else {
            Ok(())
        }
    }

    pub fn checked_slice<'a>(self, data: &'a [u8]) -> Result<&'a [u8], LimitError> {
        self.checked_len(data.len())?;
        Ok(data)
    }

    pub fn checked_count(self, count: usize, what: &'static str) -> Result<(), LimitError> {
        let limit = if what == "attributes" {
            self.max_attributes
        } else {
            self.max_nodes
        };
        if count > limit {
            Err(LimitError::TooManyItems {
                what,
                declared: count,
                limit,
            })
        } else {
            Ok(())
        }
    }

    pub fn checked_depth(self, depth: usize) -> Result<(), LimitError> {
        if depth > self.max_depth {
            Err(LimitError::TooDeep {
                limit: self.max_depth,
            })
        } else {
            Ok(())
        }
    }
}
