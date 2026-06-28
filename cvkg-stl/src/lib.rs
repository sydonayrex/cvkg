//! STL file parser — binary and ASCII formats.
//!
//! Zero external dependencies. Parses both binary and ASCII STL,
//! auto-detects format, and produces indexed mesh data compatible
//! with `cvkg_core::Mesh`.

#![warn(missing_docs)]

/// STL error types and core types.
pub mod error;

mod ascii;
mod binary;
mod detect;
mod normal;

pub use error::{detect_format, StlError, StlFormat, StlMesh};

use std::io::{Read, Seek};

/// Parse STL from any `Read + Seek` impl. Auto-detects ASCII/binary.
pub fn parse<R: Read + Seek>(reader: R) -> Result<StlMesh, StlError> {
    detect::parse(reader)
}

/// Parse STL from byte slice. Auto-detects ASCII/binary.
pub fn parse_bytes(bytes: &[u8]) -> Result<StlMesh, StlError> {
    parse(std::io::Cursor::new(bytes))
}

/// Parse with a specific format hint (skips auto-detection).
pub fn parse_with_hint<R: Read>(reader: R, hint: StlFormat) -> Result<StlMesh, StlError> {
    match hint {
        StlFormat::Binary => binary::parse(reader),
        StlFormat::Ascii => ascii::parse(reader),
    }
}
