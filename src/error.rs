use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter};
use std::io;

pub type BoxError = Box<dyn StdError>;

pub struct MigrationError {
    pub(crate) message: String,
    pub(crate) cause: Option<BoxError>,
}

impl Display for MigrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(cause) = &self.cause {
            write!(f, "{}: {}", self.message, cause)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl Debug for MigrationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MigrationError")
            .field("message", &self.message)
            .field("cause", &self.cause)
            .finish()
    }
}

impl StdError for MigrationError {
    fn cause(&self) -> Option<&dyn StdError> {
        self.cause.as_ref().map(|e| e.as_ref())
    }
}

impl From<io::Error> for MigrationError {
    fn from(err: io::Error) -> Self {
        Self {
            message: err.to_string(),
            cause: Some(Box::new(err)),
        }
    }
}

#[cfg(feature = "postgres")]
impl From<postgres::Error> for MigrationError {
    fn from(err: postgres::Error) -> Self {
        Self {
            message: err.to_string(),
            cause: Some(Box::new(err)),
        }
    }
}

#[cfg(feature = "tokio-postgres")]
impl From<tokio_postgres::Error> for MigrationError {
    fn from(err: tokio_postgres::Error) -> Self {
        Self {
            message: err.to_string(),
            cause: Some(Box::new(err)),
        }
    }
}
