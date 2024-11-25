use std::{env, ffi::CString};

use crate::{
    utils::{acquire_lock, check_refprop_error},
    RefpropError, RefpropFunctionLibrary,
};

use super::bindings;

impl RefpropFunctionLibrary {
    /// Sets the path where the fluid files are located.
    ///
    /// This function sets the directory path where REFPROP can find the necessary fluid files.
    /// The path does not need to contain the ending "/", and it can point directly to the location
    /// where the DLL is stored if a fluids subdirectory (with the corresponding fluid files) is located there.
    /// For example, `path = "C:/Program Files (x86)/REFPROP"`.
    ///
    /// # Parameters
    ///
    /// - `path`: The directory path to set for fluid files.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if the provided path contains null bytes.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error while setting the path.
    ///
    /// # References
    ///
    /// - [SETPATHdll Documentation](https://refprop-docs.readthedocs.io/en/latest/DLL/high_level.html#f/_/SETPATHdll)
    ///
    /// # Example
    ///
    /// ```rust
    /// use refprop_sys::{RefpropFunctionLibrary, RefpropError};
    ///
    /// fn main() -> Result<(), RefpropError> {
    ///     RefpropFunctionLibrary::set_path(None)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn set_path(path: Option<&str>) -> Result<(), RefpropError> {
        // Acquire the mutex lock to ensure exclusive access
        let guard = acquire_lock()?;

        // Convert Rust string to CString, ensuring no null bytes
        let c_path = CString::new(path.unwrap_or(&env::var("RPPREFIX").unwrap()))
            .map_err(|e| RefpropError::InvalidInput(format!("Path contains null byte: {}", e)))?;

        // Prepare a buffer of 255 characters, initialized to zero
        let mut buffer = [0 as libc::c_char; 255];
        let c_bytes = c_path.as_bytes_with_nul();

        // Copy up to 254 bytes into buffer, leaving space for null terminator
        for (i, &byte) in c_bytes.iter().take(254).enumerate() {
            buffer[i] = byte as libc::c_char;
        }
        // Ensure null termination
        buffer[254] = 0;

        // Call SETPATHdll
        unsafe {
            bindings::SETPATHdll(buffer.as_mut_ptr(), 255);
        }

        // Check for any errors using ERRMSGdll
        let ierr: i32 = 0;
        let mut herr = vec![0 as libc::c_char; 256];
        check_refprop_error(&guard, ierr, herr.as_mut_ptr(), herr.len() as i32)?;

        Ok(())
    }
}
