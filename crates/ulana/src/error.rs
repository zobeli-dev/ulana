use std::fmt;

/// Why a view could not load, render, or handle an event.
///
/// The messages are written to be read by a person or handed back to the
/// model that wrote the script, so it can fix it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The script does not compile.
    Syntax(String),
    /// The script raised an error while it ran.
    Runtime(String),
    /// The script used more memory than [`crate::Limits::memory`].
    Memory,
    /// The script ran longer than [`crate::Limits::time`].
    Timeout,
    /// The tree, or a value the script stored, breaks a rule.
    Invalid { path: String, message: String },
    /// The event names a frame that is no longer the latest.
    Stale { latest: u64 },
    /// The latest frame has no handler with that id.
    UnknownEvent(String),
}

impl Error {
    pub(crate) fn invalid(path: impl Into<String>, message: impl Into<String>) -> Self {
        Error::Invalid {
            path: path.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Syntax(m) => write!(f, "syntax error: {m}"),
            Error::Runtime(m) => write!(f, "runtime error: {m}"),
            Error::Memory => f.write_str("the view used more memory than it is allowed"),
            Error::Timeout => f.write_str("the view ran longer than it is allowed"),
            Error::Invalid { path, message } if path.is_empty() => f.write_str(message),
            Error::Invalid { path, message } => write!(f, "at {path}: {message}"),
            Error::Stale { latest } => {
                write!(f, "the event is for an older frame; the latest is {latest}")
            }
            Error::UnknownEvent(id) => write!(f, "the latest frame has no handler `{id}`"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;
