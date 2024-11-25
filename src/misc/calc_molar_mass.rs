use crate::{
    bindings,
    utils::{acquire_lock, validate_composition},
    RefpropError, RefpropFunctionLibrary,
};

impl RefpropFunctionLibrary {
    /// Calculates the molar mass (molecular weight) of a mixture using the `WMOLdll` function.
    ///
    /// This method computes the molar mass based on the provided composition of the mixture.
    ///
    /// **Note:** Ensure that the composition sums to 1 (within a reasonable tolerance) before calling this function.
    ///
    /// # Parameters
    ///
    /// - `z`: A slice containing the overall composition (mole fractions). Maximum of 20 components.
    ///
    /// # Returns
    ///
    /// - `WmolOutput`: A struct containing the calculated molar mass.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `z` contains more than 20 elements.
    ///     - The sum of mole fractions does not equal 1 within a specified tolerance.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - WMOLdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn calc_molar_mass(z: &[f64]) -> Result<f64, RefpropError> {
        // Validate composition slice length
        validate_composition(z)?;

        // Acquire the mutex lock to ensure exclusive access
        let _guard = acquire_lock()?;

        // Convert 'z' slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        // Initialize output buffer
        let mut wmm_out: f64 = 0.0;

        // Prepare mutable pointers for FFI
        let z_ptr = z_buffer.as_mut_ptr();
        let wmm_ptr = &mut wmm_out as *mut f64;

        // Call WMOLdll within unsafe block
        unsafe {
            bindings::WMOLdll(z_ptr, wmm_ptr);
        }

        Ok(wmm_out)
    }
}
