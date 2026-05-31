use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Template {
    Minimal,
    Dashboard,
    AiCopilot,
}

impl std::str::FromStr for Template {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dashboard" => Ok(Template::Dashboard),
            "ai" | "ai-copilot" => Ok(Template::AiCopilot),
            _ => Ok(Template::Minimal),
        }
    }
}

/// Project scaffold generator for CVKG applications.
pub struct Scaffolder {
    pub name: String,
    pub template: Template,
    pub init_git: bool,
}

impl Scaffolder {
    /// Create a new Scaffolder for a project name.
    pub fn new(name: String, template: Template, init_git: bool) -> Self {
        Self {
            name,
            template,
            init_git,
        }
    }

    /// Execute the scaffolding process.
    pub fn run(&self) -> Result<()> {
        let root = Path::new(&self.name);
        if root.exists() {
            return Err(anyhow::anyhow!("Directory already exists: {}", self.name));
        }

        println!("🛠️ Scaffolding CVKG application: {}...", self.name);

        fs::create_dir_all(root.join("src"))?;
        fs::create_dir_all(root.join("assets"))?;
        fs::create_dir_all(root.join("themes"))?;
        fs::create_dir_all(root.join(".github").join("workflows"))?;

        self.gen_cargo_toml(root)?;
        self.gen_main_rs(root)?;
        self.gen_theme_rs(root)?;
        self.gen_gitignore(root)?;
        self.gen_ci_yml(root)?;

        if self.init_git {
            println!("📦 Initializing git repository...");
            Command::new("git").arg("init").current_dir(root).output()?;
        }

        println!("✅ Successfully scaffolded CVKG project: {}", self.name);
        println!("\nNext steps:");
        println!("  cd {}", self.name);
        println!("  cvkg dev");

        Ok(())
    }

    fn gen_cargo_toml(&self, root: &Path) -> Result<()> {
        let version = env!("CARGO_PKG_VERSION");
        let content = format!(
            r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
cvkg = {{ version = "{}" }}
cvkg-core = {{ version = "{}" }}
tokio = {{ version = "1.0", features = ["full"] }}
log = "0.4"
"#,
            self.name, version, version
        );

        fs::write(root.join("Cargo.toml"), content).context("Failed to write Cargo.toml")
    }

    fn gen_main_rs(&self, root: &Path) -> Result<()> {
        let content = match self.template {
            Template::Minimal => {
                r#"use cvkg_core::View;

/// Application entry point for the minimal CVKG template.
///
/// For a full application, implement the `View` trait and
/// register it with your platform renderer.
fn main() {
    println!("CVKG Minimal Template — Replace with your app.");
    println!("See https://github.com/sydonayrex/cvkg for documentation.");
}
"#
            }
            Template::Dashboard => {
                r#"use cvkg_core::View;

/// Application entry point for the dashboard CVKG template.
///
/// For a full application, implement the `View` trait and
/// register it with your platform renderer.
fn main() {
    println!("CVKG Dashboard Template — Replace with your app.");
    println!("See https://github.com/sydonayrex/cvkg for documentation.");
}
"#
            }
            Template::AiCopilot => {
                r#"use cvkg_core::View;

/// Application entry point for the AI Copilot CVKG template.
///
/// For a full application, implement the `View` trait and
/// register it with your platform renderer.
fn main() {
    println!("CVKG AI Copilot Template — Replace with your app.");
    println!("See https://github.com/sydonayrex/cvkg for documentation.");
}
"#
            }
        };
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

    fn gen_gitignore(&self, root: &Path) -> Result<()> {
        let content = r#"/target
**/*.rs.bk
Cargo.lock
"#;
        fs::write(root.join(".gitignore"), content).context("Failed to write .gitignore")
    }

    fn gen_ci_yml(&self, root: &Path) -> Result<()> {
        let content = r#"name: CVKG CI
on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
"#;
        fs::write(root.join(".github/workflows/ci.yml"), content).context("Failed to write ci.yml")
    }
}
