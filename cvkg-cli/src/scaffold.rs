use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Project scaffold generator for CVKG applications.
pub struct Scaffolder {
    pub name: String,
}

impl Scaffolder {
    /// Create a new Scaffolder for a project name.
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// Execute the scaffolding process.
    pub fn run(&self) -> Result<()> {
        let root = Path::new(&self.name);
        if root.exists() {
            return Err(anyhow::anyhow!("Directory already exists: {}", self.name));
        }

        fs::create_dir_all(root.join("src"))?;
        fs::create_dir_all(root.join("assets"))?;
        fs::create_dir_all(root.join("themes"))?;

        self.gen_cargo_toml(root)?;
        self.gen_main_rs(root)?;
        self.gen_theme_rs(root)?;

        println!("Successfully scaffolded CVKG project: {}", self.name);
        println!("Next steps:");
        println!("  cd {}", self.name);
        println!("  cvkg dev");

        Ok(())
    }

    fn gen_cargo_toml(&self, root: &Path) -> Result<()> {
        let content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
cvkg = {{ version = "0.1.10", features = ["native"] }}
tokio = {{ version = "1.0", features = ["full"] }}
"#,
            self.name
        );

        fs::write(root.join("Cargo.toml"), content).context("Failed to write Cargo.toml")
    }

    fn gen_main_rs(&self, root: &Path) -> Result<()> {
        let content = r#"use cvkg::prelude::*;

struct App;

impl View for App {
    type Body = cvkg::core::Never;
    fn body(self) -> Self::Body { unreachable!() }

    fn render(&self, renderer: &mut dyn cvkg::core::Renderer, rect: Rect) {
        renderer.fill_rect(rect, [0.05, 0.05, 0.1, 1.0]);
        renderer.draw_text("HELLO CVKG", rect.width/2.0 - 50.0, rect.height/2.0, 32.0, [0.0, 1.0, 1.0, 1.0]);
    }
}

fn main() {
    cvkg::native::NativeRenderer::run(App);
}
"#;
        fs::write(root.join("src/main.rs"), content).context("Failed to write src/main.rs")
    }

    fn gen_theme_rs(&self, root: &Path) -> Result<()> {
        let content = r#"/// Default Niflheim Theme for CVKG
pub struct Theme {
    pub primary: [f32; 4],
    pub background: [f32; 4],
}

pub const DEFAULT_THEME: Theme = Theme {
    primary: [0.0, 1.0, 1.0, 1.0],
    background: [0.05, 0.05, 0.1, 1.0],
};
"#;
        fs::write(root.join("themes/default.rs"), content)
            .context("Failed to write themes/default.rs")
    }
}
