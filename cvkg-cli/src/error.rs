//! CLI error types for user-friendly error reporting.

use std::fmt;

/// Errors that can occur during CLI operations.
#[derive(Debug)]
pub enum CliError {
    /// File I/O error.
    Io(std::io::Error),
    /// Command execution failed {
    CommandFailed { command: String, exit_code: i32 },
    /// Invalid user input.
    InvalidInput { message: String },
    /// Crate operation error.
    CrateError { name: String, message: String },
    /// Build error with parsed cargo output.
    BuildError {
        message: String,
        file: Option<String>,
        line: Option<u32>,
        column: Option<u32>,
    },
    /// General error with message.
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(e) => write!(f, "I/O error: {}", e),
            CliError::CommandFailed { command, exit_code } => {
                write!(
                    f,
                    "Command '{}' failed with exit code {}",
                    command, exit_code
                )
            }
            CliError::InvalidInput { message } => write!(f, "Invalid input: {}", message),
            CliError::CrateError { name, message } => {
                write!(f, "Crate '{}' error: {}", name, message)
            }
            CliError::BuildError {
                message,
                file,
                line,
                column,
            } => {
                write!(f, "Build error: {}", message)?;
                if let Some(file) = file {
                    write!(f, " ({})", file)?;
                    if let Some(line) = line {
                        write!(f, ":{}", line)?;
                        if let Some(col) = column {
                            write!(f, ":{}", col)?;
                        }
                    }
                }
                Ok(())
            }
            CliError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e)
    }
}

impl From<String> for CliError {
    fn from(s: String) -> Self {
        CliError::Other(s)
    }
}

impl From<&str> for CliError {
    fn from(s: &str) -> Self {
        CliError::Other(s.to_string())
    }
}

impl From<cvkg_core::error_types::CvkgError> for CliError {
    fn from(e: cvkg_core::error_types::CvkgError) -> Self {
        CliError::Other(e.to_string())
    }
}

/// Print a user-friendly error message and exit with code 1.
pub fn exit_with_error(error: CliError) -> ! {
    use console::style;
    eprintln!("{} {}", style("❌").red(), error);
    std::process::exit(1);
}
