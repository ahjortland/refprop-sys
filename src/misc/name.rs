use std::ffi::{c_char, c_int, CStr};

use crate::{bindings, utils::acquire_lock, RefpropError, RefpropFunctionLibrary};

impl RefpropFunctionLibrary {
    /// Computes the name information for a specified component using the `NAMEdll` subroutine from REFPROP.
    ///
    /// # Parameters
    ///
    /// - `icomp`: Component number in mixture; 1 for pure fluid.
    ///
    /// # Returns
    ///
    /// - `NameOutput`: A struct containing the component name (`hnam`), long form name (`hn80`), and Chemical Abstracts Service number (`hcasn`).
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `icomp` is not a valid component number.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - NAMEdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn name(icomp: i32) -> Result<NameOutput, RefpropError> {
        // Acquire the mutex lock to ensure exclusive access
        let _lock = acquire_lock()?;

        // Initialize variables
        let mut icomp_mut = icomp as c_int;

        // Allocate buffers for output strings
        let mut hnam_buffer = [0 as c_char; 12];
        let mut hn80_buffer = [0 as c_char; 80];
        let mut hcasn_buffer = [0 as c_char; 12];

        let hnam_length = 12;
        let hn80_length = 80;
        let hcasn_length = 12;

        // Prepare mutable pointers for FFI
        let icomp_ptr = &mut icomp_mut as *mut c_int;
        let hnam_ptr = hnam_buffer.as_mut_ptr();
        let hn80_ptr = hn80_buffer.as_mut_ptr();
        let hcasn_ptr = hcasn_buffer.as_mut_ptr();
        let hnam_length = hnam_length as c_int;
        let hn80_length = hn80_length as c_int;
        let hcasn_length = hcasn_length as c_int;

        // Call NAMEdll within unsafe block
        unsafe {
            bindings::NAMEdll(
                icomp_ptr,
                hnam_ptr,
                hn80_ptr,
                hcasn_ptr,
                hnam_length,
                hn80_length,
                hcasn_length,
            );
        }

        // Convert C strings to Rust strings
        let hnam = unsafe { CStr::from_ptr(hnam_ptr) }
            .to_str()?
            .trim_end()
            .to_owned();
        let hn80 = unsafe { CStr::from_ptr(hn80_ptr) }
            .to_str()?
            .trim_end()
            .to_owned();
        let hcasn = unsafe { CStr::from_ptr(hcasn_ptr) }
            .to_str()?
            .trim_end()
            .to_owned();

        // Construct the output struct
        let output = NameOutput {
            hnam: hnam.to_owned(),
            hn80: hn80.to_owned(),
            hcasn: hcasn.to_owned(),
        };

        Ok(output)
    }

    pub fn get_filename(component: usize) -> Result<String, RefpropError> {
        let icomp = -(component as i32);
        let output = Self::name(icomp)?;

        return Ok(output.hn80);
    }
}

/// Struct representing the output of the `name` function.
///
/// Contains the component name (`hnam`), long form name (`hn80`), and Chemical Abstracts Service number (`hcasn`).
pub struct NameOutput {
    /// Component name (character*12)
    pub hnam: String,
    /// Component name - long form (character*80)
    pub hn80: String,
    /// Chemical Abstracts Service number (character*12)
    pub hcasn: String,
}

#[cfg(test)]
mod tests {
    use crate::{RefpropError, RefpropFunctionLibrary};

    #[test]
    fn test_filename() -> Result<(), RefpropError> {
        RefpropFunctionLibrary::set_path(None)?;
        RefpropFunctionLibrary::set_mixture("R457A.MIX")?;

        let name1 = RefpropFunctionLibrary::get_filename(1)?;
        let name2 = RefpropFunctionLibrary::get_filename(2)?;
        let name3 = RefpropFunctionLibrary::get_filename(3)?;
        let name4 = RefpropFunctionLibrary::get_filename(4)?;

        assert!(name4.is_empty());

        println!("{0} * {1} * {2} * {3}", name1, name2, name3, name4);

        Ok(())
    }
}
