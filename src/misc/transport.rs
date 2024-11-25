use std::ffi::{c_char, c_int};

use crate::{
    bindings,
    utils::{acquire_lock, check_refprop_error, validate_composition},
    RefpropError, RefpropFunctionLibrary,
};

use super::TransportOutput;

impl RefpropFunctionLibrary {
    /// Computes the transport properties (viscosity and thermal conductivity) as functions of temperature, density, and composition using the `TRNPRPdll` function.
    ///
    /// **Warning:**
    ///
    /// Do NOT call this routine for two-phase states. If near the phase boundary, it may return a metastable state or nonsensical results.
    /// The value of `q` returned from flash routines will indicate a two-phase state by returning a value between 0 and 1.
    /// In such situations, transport properties can only be calculated for the saturated liquid and vapor states.
    ///
    /// # Parameters
    ///
    /// - `T`: Temperature [K]
    /// - `D`: Molar density [mol/L]
    /// - `z`: Composition array (slice of mole fractions). Maximum of 20 components.
    ///
    /// # Returns
    ///
    /// - `TrnprpOutput`: A struct containing the calculated viscosity and thermal conductivity.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `z` contains more than 20 elements.
    ///     - The sum of mole fractions in `z` does not equal 1 within a specified tolerance.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - TRNPRPdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn transport_properties(
        T: f64,
        D: f64,
        z: &[f64],
    ) -> Result<TransportOutput, RefpropError> {
        // Validate composition slice
        validate_composition(z)?;

        // Acquire the mutex lock to ensure exclusive access
        let lock = acquire_lock()?;

        // Convert 'z' slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        // Initialize output buffers
        let mut eta: f64 = 0.0;
        let mut tcx: f64 = 0.0;
        let mut ierr: i32 = 0;
        let herr_buffer = [0 as c_char; 255]; // Fixed-size array for error messages

        let mut T = T;
        let mut D = D;

        // Prepare mutable pointers for FFI
        let T_ptr = &mut T as *mut f64; // REFPROP may modify T
        let D_ptr = &mut D as *mut f64; // REFPROP may modify D
        let z_ptr = z_buffer.as_mut_ptr();
        let eta_ptr = &mut eta as *mut f64;
        let tcx_ptr = &mut tcx as *mut f64;
        let ierr_ptr = &mut ierr as *mut c_int;
        let herr_ptr = herr_buffer.as_ptr() as *mut c_char;
        let herr_length = 255 as c_int;

        // Call TRNPRPdll within unsafe block
        unsafe {
            bindings::TRNPRPdll(
                T_ptr,
                D_ptr,
                z_ptr,
                eta_ptr,
                tcx_ptr,
                ierr_ptr,
                herr_ptr,
                herr_length,
            );
        }

        // Check for errors using the helper function, passing the lock
        check_refprop_error(&lock, ierr, herr_ptr, herr_length)?;

        // Construct the output struct with the retrieved properties
        let output = TransportOutput { eta, tcx };

        Ok(output)
    }
}
