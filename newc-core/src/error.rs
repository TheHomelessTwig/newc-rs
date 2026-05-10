use thiserror::Error;

#[derive(Debug, Error)]
pub enum NewcError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Project '{0}' already exists")]
    ProjectExists(String),

    #[error("Not a newc project (missing src/main.c, include/, or Makefile)")]
    NotAProject,

    #[error("Module '{0}' not found")]
    ModuleNotFound(String),

    #[error("Module '{0}' already exists")]
    ModuleExists(String),

    #[error("'{0}' is not a valid C identifier (use letters, digits, underscores; must not start with a digit)")]
    InvalidName(String),

    #[error("No modules found")]
    NoModules,

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, NewcError>;
