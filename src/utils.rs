use std::{
    ffi::{c_char, c_int, CStr},
    sync::{Mutex, MutexGuard},
};

use crate::{bindings, RefpropError, REFPROP_MUTEX};

pub(crate) fn acquire_lock<'a>() -> Result<MutexGuard<'a, ()>, RefpropError> {
    let lock = REFPROP_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| RefpropError::MutexPoisoned)?;

    Ok(lock)
}

pub(crate) fn validate_composition(z: &[f64]) -> Result<(), RefpropError> {
    if z.len() > 20 {
        return Err(RefpropError::InvalidInput(
            "Composition slice 'z' length exceeds 20.".to_string(),
        ));
    }
    let sum_z: f64 = z.iter().sum();
    if (sum_z - 1.0).abs() > 1e-6 {
        return Err(RefpropError::InvalidInput(format!(
            "Sum of mole fractions in 'z' is {}, which does not equal 1 within tolerance.",
            sum_z
        )));
    }
    Ok(())
}

/// Checks the REFPROP error code and retrieves the error message if an error is present.
///
/// # Parameters
///
/// - `guard`: Reference to the mutex guard ensuring exclusive access to REFPROP.
/// - `ierr`: Error code returned by the REFPROP function.
/// - `herr_ptr`: Pointer to the error message buffer.
/// - `herr_length`: Length of the error message buffer.
///
/// # Returns
///
/// - `Ok(())` if no error occurred.
/// - `Err(RefpropError::CalculationError)` with the error message if an error is detected.
/// - `Err(RefpropError::Utf8Error)` if the error message cannot be converted to UTF-8.
///
/// # Safety
///
/// This function contains unsafe code due to FFI interactions.
/// It should only be called with valid pointers and buffer lengths as per REFPROP's specifications.
pub(crate) fn check_refprop_error(
    _guard: &MutexGuard<()>,
    ierr: i32,
    herr_ptr: *mut c_char,
    herr_length: c_int,
) -> Result<(), RefpropError> {
    if ierr != 0 {
        // Initialize a mutable variable to store the error code from ERRMSGdll
        let mut ierr_errmsg: i32 = 0;

        unsafe {
            // Call ERRMSGdll to retrieve the error message
            bindings::ERRMSGdll(&mut ierr_errmsg as *mut i32, herr_ptr, herr_length);
        }

        // Convert the C-style error message to a Rust String
        let error_message = unsafe {
            // Ensure that herr_ptr points to a valid C-string
            CStr::from_ptr(herr_ptr)
                .to_str()
                .map_err(RefpropError::Utf8Error)?
                .to_string()
        };

        // Return the error
        Err(RefpropError::CalculationError(error_message))
    } else {
        Ok(())
    }
}
