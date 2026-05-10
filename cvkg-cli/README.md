# cvkg-cli

**cvkg-cli** provides command-line tools for CVKG development and project management.

## What This Crate Does

- Provides `scaffold` command for new project creation
- Provides `build` command for building CVKG applications
- Provides `serve` command for development server

## What This Crate Does NOT Do

- Does not provide rendering functionality
- Does not provide UI components
- Does not handle application deployment

## Public API Overview

### Commands

```rust
use cvkg_cli::{Commands, run};

pub enum Commands {
    Scaffold { name: String, template: Option<String> },
    Build { release: bool, target: Option<String> },
    Serve { port: u16, open: bool },
}
pub fn run(commands: Commands) -> Result<(), CliError>;
```

### Scaffold

```rust
/// Create a new CVKG project
run(Commands::Scaffold { name: "my-app".to_string(), template: None });
```

### Build

```rust
/// Build a CVKG application
run(Commands::Build { release: true, target: None });
```

### Serve

```rust
/// Start development server
run(Commands::Serve { port: 3000, open: true });
```

## Usage Example

```bash
# Create new project
cvkg scaffold my-app

# Build for production
cvkg build --release

# Start development server
cvkg serve --port 3000
```

## Known Limitations

- Templates are limited to basic project structure
- Build command requires existing Cargo.toml
- Serve command does not support HTTPS