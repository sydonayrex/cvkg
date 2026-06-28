// Format auto-detection — implemented in P2
use crate::detect_format;
use crate::error::{StlError, StlFormat, StlMesh};
use crate::{ascii, binary};
use std::io::{Read, Seek, SeekFrom};

pub fn parse<R: Read + Seek>(mut reader: R) -> Result<StlMesh, StlError> {
    let mut header = [0u8; 80];
    let n = std::io::Read::read(&mut reader, &mut header)?;
    if n < 80 {
        return Err(StlError::Truncated);
    }

    let format = detect_format(&header).unwrap_or(StlFormat::Binary);

    // Reset to beginning for full parse
    reader.seek(SeekFrom::Start(0))?;

    match format {
        StlFormat::Ascii => {
            let result = ascii::parse(&mut reader);
            match result {
                Ok(mesh) => Ok(mesh),
                // Only fall back to binary if data isn't ASCII at all
                Err(StlError::NotAscii) => {
                    reader.seek(SeekFrom::Start(0))?;
                    binary::parse(reader)
                }
                Err(e) => Err(e),
            }
        }
        StlFormat::Binary => binary::parse(reader),
    }
}
