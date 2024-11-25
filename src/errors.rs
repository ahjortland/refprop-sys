use std::str::Utf8Error;
use thiserror::Error;

/// Custom error type for REFPROP interactions.
#[derive(Error, Debug)]
pub enum RefpropError {
    /// Represents errors that occur during the initialization of REFPROP.
    #[error("Initialization failed: {0}")]
    InitializationError(String),

    /// Represents errors that occur during REFPROP calculations.
    #[error("Calculation failed: {0}")]
    CalculationError(String),

    /// Represents invalid input errors, such as incorrect parameters or data.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Represents errors that occur while converting C strings to Rust strings.
    #[error("UTF-8 conversion error: {0}")]
    Utf8Error(#[from] Utf8Error),

    /// Represents errors when the mutex protecting REFPROP is poisoned.
    #[error("Mutex was poisoned")]
    MutexPoisoned,

    /// Represents any unknown or unexpected errors.
    #[error("Unknown error: {0}")]
    UnknownError(String),
}
