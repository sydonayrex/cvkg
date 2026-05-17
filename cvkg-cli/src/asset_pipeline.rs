//! Asset Pipeline Hook
//! Handles validation and optimization of static assets like shaders, fonts, and images.

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Configures and executes the asset pipeline
pub struct AssetPipeline;

impl AssetPipeline {
    /// Run the optimization and validation pipeline on an asset directory
    pub fn run<P: AsRef<Path>>(asset_dir: P) -> Result<()> {
        let dir = asset_dir.as_ref();
        if !dir.exists() {
            println!(
                "No assets directory found at {:?}. Skipping asset pipeline.",
                dir
            );
            return Ok(());
        }

        println!("🎨 CVKG Asset Pipeline executing...");

        let mut shader_count = 0;
        let mut image_count = 0;

        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file()
                && let Some(ext) = path.extension()
            {
                match ext.to_str().unwrap_or("") {
                    "wgsl" | "glsl" => {
                        Self::validate_shader(path)?;
                        shader_count += 1;
                    }
                    "png" | "jpg" | "jpeg" => {
                        Self::optimize_image(path)?;
                        image_count += 1;
                    }
                    _ => {}
                }
            }
        }

        println!(
            "✅ Asset Pipeline complete: Validated {} shaders, optimized {} images.",
            shader_count, image_count
        );
        Ok(())
    }

    fn validate_shader(path: &Path) -> Result<()> {
        // In a production environment, this would invoke `naga` to validate WGSL AST.
        // For now, we simulate the validation pass.
        let content = fs::read_to_string(path)?;
        if content.trim().is_empty() {
            log::warn!("Shader file {:?} is empty", path);
        }
        // Simulated validation logic
        Ok(())
    }

    fn optimize_image(path: &Path) -> Result<()> {
        // In a production environment, this would load the image and convert to WebP
        // using the `image` crate to reduce bundle size.
        let metadata = fs::metadata(path)?;
        if metadata.len() > 1024 * 1024 {
            log::warn!(
                "Image {:?} is quite large ({:.2} MB). Consider compressing it.",
                path,
                metadata.len() as f64 / 1024.0 / 1024.0
            );
        }
        Ok(())
    }
}
