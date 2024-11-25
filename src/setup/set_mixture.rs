use std::ffi::CString;

use crate::{
    bindings,
    utils::{acquire_lock, check_refprop_error},
    RefpropError, RefpropFunctionLibrary,
};

impl RefpropFunctionLibrary {
    /// Sets the mixture for the REFPROP library.
    ///
    /// This function calls the `SETMIXTUREdll` routine to load a predefined mixture and return its
    /// composition in the mole fraction array `z`. If `ierr` is non-zero, an error occurred.
    ///
    /// # Parameters
    ///
    /// - `mixture_name`: A string slice containing the name of the mixture file. The `.mix`
    ///   extension is optional, and a full path may be included if necessary.
    ///
    /// # Returns
    ///
    /// - `Vec<f64>`: A vector containing the mole fractions of the mixture components.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if the `mixture_name` contains null bytes.
    /// - Returns `RefpropError::CalculationError` if the REFPROP library fails to load the mixture.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use refprop_sys::{RefpropFunctionLibrary, RefpropError};
    ///
    /// fn main() -> Result<(), RefpropError> {
    ///     let _ = RefpropFunctionLibrary::set_path(None);
    ///
    ///     // Load a predefined mixture and get its composition
    ///     let composition = RefpropFunctionLibrary::set_mixture("AIR.MIX")?;
    ///     println!("Mixture composition: {:?}", composition);
    ///     Ok(())
    /// }
    /// ```
    pub fn set_mixture(mixture_name: &str) -> Result<Vec<f64>, RefpropError> {
        // Acquire the mutex lock to ensure exclusive access
        let guard = acquire_lock()?;

        // Convert the Rust string to a C-compatible CString
        let c_mixture_name = CString::new(mixture_name)
            .map_err(|e| RefpropError::InvalidInput(format!("Invalid mixture name: {}", e)))?;

        // Allocate a buffer for the mole fractions (up to 20 components)
        let mut z: [f64; 20] = [0.0; 20];

        // Prepare the error flag
        let mut ierr: i32 = 0;

        // Call the SETMIXTUREdll function within an unsafe block
        unsafe {
            bindings::SETMIXTUREdll(
                c_mixture_name.as_ptr() as *mut libc::c_char,
                z.as_mut_ptr(),
                &mut ierr as *mut i32,
                c_mixture_name.to_bytes_with_nul().len() as i32,
            );
        }

        // Define buffer sizes as per REFPROP's documentation
        const HERR_LENGTH: usize = 255;
        let mut herr_buffer = vec![0 as libc::c_char; HERR_LENGTH];
        let herr_ptr = herr_buffer.as_mut_ptr();
        let herr_length = HERR_LENGTH as i32;

        check_refprop_error(&guard, ierr, herr_ptr, herr_length)?;

        // Convert the composition array to a Vec and return it
        Ok(z.into_iter()
            .take_while(|&zi| zi > 0.0)
            .collect::<Vec<f64>>())
    }
}
