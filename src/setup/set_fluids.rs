use std::ffi::CString;

use crate::{
    bindings,
    utils::{acquire_lock, check_refprop_error},
    RefpropError, RefpropFunctionLibrary,
};

impl RefpropFunctionLibrary {
    /// Sets the fluids for the REFPROP library.
    ///
    /// This function calls the `SETFLUIDSdll` routine to specify the fluids used in calculations.
    /// For a pure fluid, `fluids` should contain the name of the fluid file.
    /// For a mixture, `fluids` should contain the names of the constituent fluid files separated by
    /// a `|`, `;`, or `*`. To load a predefined mixture, use the `set_mixture` method instead.
    ///
    /// **Examples:**
    ///
    /// - Load argon as a pure fluid: "ARGON"
    /// - Load a mixture of nitrogen, argon, and oxygen: "FLUIDS/NITROGEN.FLD|FLUIDS/ARGON.FLD|FLUIDS/OXYGEN.FLD|"
    /// - Load the air mixture from a pseudo-pure file: "AIR.PPF"
    /// - Load a mixture using asterisk separators: "methane * ethane * propane * butane"
    ///
    /// # Parameters
    ///
    /// - `fluids`: A string slice containing fluid file names separated by `|`, `;`, or `*`.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if the provided `fluids` string contains null bytes.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error while setting the fluids.
    ///
    /// # References
    ///
    /// - [SETFLUIDSdll Documentation](https://refprop-docs.readthedocs.io/en/latest/DLL/high_level.html#f/_/SETFLUIDSdll)
    ///
    /// # Example
    ///
    /// ```rust
    /// use refprop_sys::{RefpropFunctionLibrary, RefpropError};
    ///
    /// fn main() -> Result<(), RefpropError> {
    ///     let _ = RefpropFunctionLibrary::set_path(None);
    ///
    ///     // Set the fluids (e.g., load argon as a pure fluid)
    ///     RefpropFunctionLibrary::set_fluids("ARGON")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn set_fluids(fluids: &str) -> Result<(), RefpropError> {
        // Acquire the mutex lock to ensure exclusive access
        let guard = acquire_lock()?;

        // Convert Rust string to CString, ensuring no null bytes
        let c_fluids = CString::new(fluids).map_err(|e| {
            RefpropError::InvalidInput(format!("Fluids string contains null byte: {}", e))
        })?;

        // Define the maximum buffer size as per REFPROP's default
        const MAX_FLUIDS_LENGTH: usize = 10000;

        // Initialize a buffer with zeros
        let mut buffer = vec![0 as libc::c_char; MAX_FLUIDS_LENGTH];

        // Get the bytes of the CString including the null terminator
        let c_bytes = c_fluids.as_bytes_with_nul();

        // Determine the number of bytes to copy (ensure it doesn't exceed MAX_FLUIDS_LENGTH - 1)
        let bytes_to_copy = if c_bytes.len() > MAX_FLUIDS_LENGTH {
            MAX_FLUIDS_LENGTH - 1
        } else {
            c_bytes.len()
        };

        // Copy the bytes into the buffer, casting &[u8] to &[i8]
        buffer[..bytes_to_copy]
            .copy_from_slice(unsafe { &*(c_bytes as *const [u8] as *const [i8]) });

        // Ensure the buffer is null-terminated
        buffer[bytes_to_copy] = 0;

        // Prepare the length as i32
        let hfld_length = MAX_FLUIDS_LENGTH as i32;

        // Prepare the ierr variable
        let mut ierr: i32 = 0;

        // Define buffer sizes as per REFPROP's documentation
        const HERR_LENGTH: usize = 255;
        let mut herr_buffer = vec![0 as libc::c_char; HERR_LENGTH];
        let herr_ptr = herr_buffer.as_mut_ptr();
        let herr_length = HERR_LENGTH as i32;

        // Call SETFLUIDSdll within an unsafe block
        unsafe {
            bindings::SETFLUIDSdll(buffer.as_mut_ptr(), &mut ierr as *mut i32, hfld_length);
        }

        check_refprop_error(&guard, ierr, herr_ptr, herr_length)?;

        Ok(())
    }
}
