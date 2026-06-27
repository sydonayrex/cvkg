//! Raster export -- Exports rendered views to PNG and GIF formats.
//!
//! Delegates encoding to the `cvkg-export-raster` crate.
//! No FFmpeg dependency.

use std::path::PathBuf;

/// Export configuration for raster output.
pub struct RasterExport {
    /// Output format
    pub format: String,
    /// Output file path
    pub output: PathBuf,
    /// Number of frames (for gif)
    pub frames: u16,
    /// Frames per second (for gif)
    pub fps: u16,
}

impl RasterExport {
    /// Create a new raster export configuration.
    pub fn new(format: String, output: PathBuf, frames: u16, fps: u16) -> Self {
        Self {
            format,
            output,
            frames,
            fps,
        }
    }

    /// Execute the export.
    pub fn execute(&self) -> Result<(), String> {
        println!(
            "Raster export: format={}, output={}, frames={}, fps={}",
            self.format,
            self.output.display(),
            self.frames,
            self.fps
        );

        if self.format == "png" {
            self.export_png()
        } else if self.format == "gif" {
            self.export_gif()
        } else {
            Err(format!("Unsupported format: {}", self.format))
        }
    }

    fn export_png(&self) -> Result<(), String> {
        // Create a test pattern using cvkg-export-raster
        let frame = cvkg_export_raster::CapturedFrame {
            width: 200,
            height: 200,
            rgba: vec![128u8; 200 * 200 * 4],
        };
        let png_bytes = cvkg_export_raster::encode_png(&frame)?;
        std::fs::write(&self.output, png_bytes)
            .map_err(|e| format!("Failed to write PNG: {}", e))?;
        println!("Wrote test PNG to {}", self.output.display());
        Ok(())
    }

    fn export_gif(&self) -> Result<(), String> {
        let frames: Vec<cvkg_export_raster::CapturedFrame> = (0..self.frames)
            .map(|_| cvkg_export_raster::CapturedFrame {
                width: 200,
                height: 200,
                rgba: vec![128u8; 200 * 200 * 4],
            })
            .collect();
        let gif_bytes = cvkg_export_raster::encode_gif(&frames, self.fps)?;
        std::fs::write(&self.output, gif_bytes)
            .map_err(|e| format!("Failed to write GIF: {}", e))?;
        println!(
            "Wrote test GIF ({} frames) to {}",
            self.frames,
            self.output.display()
        );
        Ok(())
    }
}
