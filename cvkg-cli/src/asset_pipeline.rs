//! Asset Pipeline Hook
//! Handles validation and optimization of static assets like shaders, fonts, and images.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Configures and executes the asset pipeline.
pub struct AssetPipeline;

impl AssetPipeline {
    /// Run the optimization and validation pipeline on an asset directory.
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

        let entries: Vec<_> = walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();

        let total = entries.len();
        let pb = indicatif::ProgressBar::new(total as u64);
        pb.set_style(
            indicatif::ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        let mut shader_count = 0;
        let mut image_count = 0;
        let mut error_count = 0;

        for entry in &entries {
            let path = entry.path();
            pb.set_message(
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            );

            if let Some(ext) = path.extension() {
                match ext.to_str().unwrap_or("") {
                    "wgsl" => {
                        if let Err(e) = Self::validate_shader(path) {
                            log::warn!("Shader validation failed for {:?}: {}", path, e);
                            error_count += 1;
                        }
                        shader_count += 1;
                    }
                    "glsl" => {
                        // GLSL support requires additional naga features
                        if let Err(e) = Self::validate_shader(path) {
                            log::warn!("Shader validation for {:?}: {}", path, e);
                            error_count += 1;
                        }
                        shader_count += 1;
                    }
                    "png" | "jpg" | "jpeg" => {
                        if let Err(e) = Self::optimize_image(path) {
                            log::warn!("Image optimization failed for {:?}: {}", path, e);
                            error_count += 1;
                        }
                        image_count += 1;
                    }
                    _ => {}
                }
            }
            pb.inc(1);
        }

        pb.finish_with_message(format!(
            "✅ Asset Pipeline complete: {} shaders validated, {} images processed{}",
            shader_count,
            image_count,
            if error_count > 0 {
                format!(", {} errors", error_count)
            } else {
                String::new()
            }
        ));
        Ok(())
    }

    /// Validate a WGSL shader using naga parser and validator.
    fn validate_shader(path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read shader file: {:?}", path))?;

        if content.trim().is_empty() {
            log::warn!("Shader file {:?} is empty", path);
            return Ok(());
        }

        // Parse using naga's WGSL frontend
        let module = naga::front::wgsl::parse_str(&content)
            .map_err(|e| anyhow::anyhow!("Shader parse error in {:?}: {}", path, e))?;

        // Validate the parsed module
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        );
        validator
            .validate(&module)
            .map_err(|e| anyhow::anyhow!("Shader validation failed in {:?}: {}", path, e))?;

        Ok(())
    }

    /// Optimize an image file -- verify it can be decoded and warn if large.
    fn optimize_image(path: &Path) -> Result<()> {
        let metadata = fs::metadata(path)
            .with_context(|| format!("Failed to read image metadata: {:?}", path))?;
        let size_mb = metadata.len() as f64 / 1024.0 / 1024.0;

        if metadata.len() > 1024 * 1024 {
            log::warn!(
                "Image {:?} is {:.2} MB. Consider compressing it to reduce bundle size.",
                path,
                size_mb
            );
        }

        // Verify the image can be decoded
        match image::open(path) {
            Ok(img) => {
                log::debug!(
                    "Image {:?} verified ({}x{}, {:?})",
                    path,
                    img.width(),
                    img.height(),
                    img.color()
                );
                Ok(())
            }
            Err(e) => {
                anyhow::bail!("Failed to decode image {:?}: {}", path, e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    fn write_temp_file(name: &str, content: &str) -> (PathBuf, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("Failed to create test dir");
        let path = dir.path().join(name);
        std::fs::write(&path, content).expect("Failed to write test file");
        (path, dir)
    }

    #[test]
    fn test_validate_shader_valid_wgsl() {
        let (path, _dir) = write_temp_file(
            "test.wgsl",
            r#"
@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    let pos = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 3.0, -1.0),
        vec2<f32>(-1.0,  3.0),
    );
    return vec4<f32>(pos[idx], 0.0, 1.0);
}
"#,
        );

        let result = AssetPipeline::validate_shader(&path);
        assert!(
            result.is_ok(),
            "Valid WGSL shader should pass validation: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_validate_shader_invalid() {
        let (path, _dir) = write_temp_file("invalid.wgsl", "this is not valid wgsl {{{");

        let result = AssetPipeline::validate_shader(&path);
        assert!(result.is_err(), "Invalid WGSL should fail validation");
    }

    #[test]
    fn test_validate_shader_empty() {
        let (path, _dir) = write_temp_file("empty.wgsl", "   \n  \n");

        let result = AssetPipeline::validate_shader(&path);
        assert!(result.is_ok(), "Empty shader should pass (with warning)");
    }

    #[test]
    fn test_optimize_image_valid_png() {
        let dir = tempfile::tempdir().expect("Failed to create temp dir");
        let path = dir.path().join("test.png");
        let img = image::RgbaImage::from_raw(1, 1, vec![255, 0, 0, 255]).unwrap();
        img.save(&path).unwrap();

        let result = AssetPipeline::optimize_image(&path);
        assert!(result.is_ok(), "Valid PNG should pass: {:?}", result.err());
    }

    #[test]
    fn test_optimize_image_invalid() {
        let (path, _dir) = write_temp_file("fake.png", "this is not a real png file");

        let result = AssetPipeline::optimize_image(&path);
        assert!(result.is_err(), "Invalid image should fail");
    }
}
