use std::fmt;

use crate::scope::CToolScope;

pub type CToolResult<T> = Result<T, CToolError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CToolError {
    ScopeDenied {
        scope: CToolScope,
        operation: &'static str,
    },
    UnsupportedScope {
        scope: CToolScope,
        operation: &'static str,
    },
    OutOfScope {
        path: String,
        operation: &'static str,
    },
    InvalidInput(String),
    Serialization(String),
    Io(String),
    ToolNotFound {
        name: String,
    },
}

impl fmt::Display for CToolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CToolError::ScopeDenied { scope, operation } => {
                write!(f, "CTool {operation} denied by CToolScope: {scope}")
            }
            CToolError::UnsupportedScope { scope, operation } => {
                write!(f, "CTool {operation} does not support CToolScope: {scope}")
            }
            CToolError::OutOfScope { path, operation } => {
                write!(
                    f,
                    "CTool {operation} path is outside the allowed scope: {path}"
                )
            }
            CToolError::InvalidInput(message) => {
                write!(f, "CTool invalid input: {message}")
            }
            CToolError::Serialization(message) => {
                write!(f, "CTool serialization error: {message}")
            }
            CToolError::Io(message) => {
                write!(f, "CTool IO error: {message}")
            }
            CToolError::ToolNotFound { name } => {
                write!(f, "CTool not found: {name}")
            }
        }
    }
}

impl std::error::Error for CToolError {}

impl From<std::io::Error> for CToolError {
    fn from(error: std::io::Error) -> Self {
        CToolError::Io(error.to_string())
    }
}

impl From<serde_json::Error> for CToolError {
    fn from(error: serde_json::Error) -> Self {
        CToolError::Serialization(error.to_string())
    }
}
