use std::ffi::CString;

use crate::{
    bindings,
    utils::{acquire_lock, check_refprop_error},
    RefpropError, RefpropFunctionLibrary,
};

/// Represents the flags for the `get_enum` method.
#[derive(Debug, Clone, Copy)]
pub enum GetEnumFlag {
    /// Check all strings possible.
    AllStrings = 0,
    /// Check strings for property units only (e.g., SI, English, etc.).
    UnitsOnly = 1,
    /// Check property strings and those in `PropertiesAnd3` only.
    UnitsAndTrivial = 2,
    /// Check property strings only that are not functions of T and D.
    TrivialOnly = 3,
}

impl GetEnumFlag {
    /// Converts the `GetEnumFlag` enum to its corresponding integer value.
    fn as_i32(self) -> i32 {
        self as i32
    }
}

impl RefpropFunctionLibrary {
    /// Translates a string of uppercase letters into an enumerated integer value.
    ///
    /// This function optimizes REFPROP property calculations by converting strings into
    /// integer flags, reducing the overhead of repeated string comparisons.
    ///
    /// # Parameters
    ///
    /// - `flag`: Specifies the type of enumeration to perform.
    /// - `enum_str`: A string slice containing uppercase letters representing the desired enumeration.
    ///
    /// # Returns
    ///
    /// - `iEnum`: An integer representing the enumerated value corresponding to `enum_str`.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if `enum_str` contains null bytes.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during enumeration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use refprop_sys::{RefpropFunctionLibrary, GetEnumFlag, RefpropError};
    ///
    /// fn main() -> Result<(), RefpropError> {
    ///     // Translate a property unit string to its enumerated value
    ///     let enum_value = RefpropFunctionLibrary::get_enum(GetEnumFlag::UnitsOnly, "SI")?;
    ///     println!("Enumerated Value: {}", enum_value);
    ///     Ok(())
    /// }
    /// ```
    pub fn get_enum(flag: GetEnumFlag, enum_str: &str) -> Result<i32, RefpropError> {
        // Acquire the mutex lock to ensure exclusive access
        let guard = acquire_lock()?;

        // Convert Rust string to CString, ensuring no null bytes
        let c_enum_str = CString::new(enum_str).map_err(|e| {
            RefpropError::InvalidInput(format!("Enum string contains null byte: {}", e))
        })?;

        // Define the default buffer size as per REFPROP's documentation
        const HENUM_LENGTH: usize = 255;
        const HERR_LENGTH: usize = 255;

        // Initialize buffers for hEnum and herr
        let mut h_enum_buffer = vec![0 as libc::c_char; HENUM_LENGTH];
        let mut herr_buffer = vec![0 as libc::c_char; HERR_LENGTH];
        let mut i_enum: i32 = 0;
        let mut ierr: i32 = 0;

        // Determine the number of bytes to copy (ensure it doesn't exceed HENUM_LENGTH - 1)
        let c_enum_bytes = c_enum_str.as_bytes_with_nul();
        let bytes_to_copy = if c_enum_bytes.len() > HENUM_LENGTH {
            HENUM_LENGTH - 1
        } else {
            c_enum_bytes.len()
        };

        // Copy the bytes into the hEnum buffer, casting &[u8] to &[i8]
        h_enum_buffer[..bytes_to_copy]
            .copy_from_slice(unsafe { &*(c_enum_bytes as *const [u8] as *const [i8]) });
        // Ensure null termination
        h_enum_buffer[bytes_to_copy] = 0;

        // Prepare the lengths as i32
        let h_enum_length = HENUM_LENGTH as i32;
        let herr_length = HERR_LENGTH as i32;

        // Call GETENUMdll within an unsafe block
        unsafe {
            bindings::GETENUMdll(
                &mut flag.as_i32() as *mut i32,
                h_enum_buffer.as_mut_ptr(),
                &mut i_enum as *mut i32,
                &mut ierr as *mut i32,
                herr_buffer.as_mut_ptr(),
                h_enum_length,
                herr_length,
            );
        }

        // Check for errors
        check_refprop_error(&guard, ierr, herr_buffer.as_mut_ptr(), herr_length)?;

        Ok(i_enum)
    }
}
