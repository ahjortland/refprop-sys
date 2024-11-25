use crate::{
    bindings,
    utils::{acquire_lock, check_refprop_error, validate_composition},
    RefpropError, RefpropFunctionLibrary,
};

impl RefpropFunctionLibrary {
    /// Calculates the phase boundary and critical points for a mixture at a given composition using the `SATSPLNdll` function.
    ///
    /// This method computes the phase boundary of a mixture and identifies critical points such as the cricondentherm
    /// and cricondenbar based on the provided composition.
    ///
    /// **Note:** This routine calculates phase boundaries and critical points but does not directly return these values.
    /// To retrieve specific phase boundary data and critical points, additional methods or bindings would be required.
    ///
    /// # Parameters
    ///
    /// - `z`: A slice containing the overall composition (mole fractions). Maximum of 20 components.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `z` contains more than 20 elements.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - SATSPLNdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn satspln(z: &[f64]) -> Result<(), RefpropError> {
        // Validate composition slice length
        validate_composition(z)?;

        // Acquire the mutex lock to ensure exclusive access
        let guard = acquire_lock()?;

        // Convert 'z' slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        // Define buffer sizes as per REFPROP's documentation
        const HERR_LENGTH: usize = 255;

        // Initialize output buffers
        let mut ierr: i32 = 0;
        let mut herr_buffer = vec![0 as libc::c_char; HERR_LENGTH];

        // Prepare mutable pointers for FFI
        let z_ptr = z_buffer.as_mut_ptr();
        let ierr_ptr = &mut ierr as *mut i32;
        let herr_ptr = herr_buffer.as_mut_ptr();
        let herr_length = HERR_LENGTH as i32;

        // Call SATSPLNdll within unsafe block
        unsafe {
            bindings::SATSPLNdll(z_ptr, ierr_ptr, herr_ptr, herr_length);
        }

        // Check ierr for errors
        check_refprop_error(&guard, ierr, herr_ptr, herr_length)?;

        Ok(())
    }
}
