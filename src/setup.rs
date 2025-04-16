mod purefld;
mod satspln;
mod set_fluids;
mod set_mixture;
mod set_path;

use std::{
    ffi::{CStr, CString},
    sync::Mutex,
};

use crate::{bindings, errors::RefpropError, RefpropFunctionLibrary, Units, REFPROP_MUTEX};

/// Represents the critical parameters calculated by the `crit_p` method.
#[derive(Debug, Clone)]
pub struct CriticalParameters {
    /// Critical temperature [K]
    pub Tc: f64,
    /// Critical pressure [kPa]
    pub Pc: f64,
    /// Critical density [mol/L]
    pub Dc: f64,
}

impl RefpropFunctionLibrary {
    /// Calculates the critical parameters of a mixture using the `CRITPdll` function.
    ///
    /// This method computes the critical temperature (`Tc`), critical pressure (`Pc`), and critical
    /// density (`Dc`) of a mixture based on its composition.
    ///
    /// **Note:** The critical parameters are estimates based on polynomial fits to the binary critical
    /// lines. For mixtures with three or more components, combining rules are applied to the constituent
    /// binaries.
    ///
    /// # Parameters
    ///
    /// - `z`: A slice containing the overall composition (mole fractions). Maximum of 20 components.
    ///
    /// # Returns
    ///
    /// - `CriticalParameters`: A struct containing the critical temperature, pressure, and density.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `z` contains more than 20 elements.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use refprop_sys::{RefpropFunctionLibrary, RefpropError};
    ///
    /// fn main() -> Result<(), RefpropError> {
    ///     // Define composition (pure component)
    ///     
    ///     let z = vec![1.0];
    ///
    ///     // Calculate critical parameters
    ///     let critical_params = RefpropFunctionLibrary::crit_p(&z)?;
    ///
    ///     // Access the calculated critical parameters
    ///     println!("Critical Temperature: {} K", critical_params.Tc);
    ///     println!("Critical Pressure: {} kPa", critical_params.Pc);
    ///     println!("Critical Density: {} mol/L", critical_params.Dc);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - CRITPdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn critical_parameters(z: &[f64]) -> Result<CriticalParameters, RefpropError> {
        // Validate composition slice length
        if z.len() > 20 {
            return Err(RefpropError::InvalidInput(
                "Composition slice 'z' length exceeds 20.".to_string(),
            ));
        }

        // Acquire the mutex lock to ensure exclusive access
        let _lock = REFPROP_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .map_err(|_| RefpropError::MutexPoisoned)?;

        // Convert 'z' slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        // Define buffer sizes as per REFPROP's documentation
        const HERR_LENGTH: usize = 255;

        // Initialize output buffers
        let mut Tc_out: f64 = 0.0;
        let mut Pc_out: f64 = 0.0;
        let mut Dc_out: f64 = 0.0;
        let mut ierr: i32 = 0;
        let mut herr_buffer = vec![0 as libc::c_char; HERR_LENGTH];

        // Prepare mutable pointers for FFI
        let z_ptr = z_buffer.as_mut_ptr();
        let Tc_ptr = &mut Tc_out as *mut f64;
        let Pc_ptr = &mut Pc_out as *mut f64;
        let Dc_ptr = &mut Dc_out as *mut f64;
        let ierr_ptr = &mut ierr as *mut i32;
        let herr_ptr = herr_buffer.as_mut_ptr();
        let herr_length = HERR_LENGTH as i32;

        // Call CRITPdll within unsafe block
        unsafe {
            bindings::CRITPdll(
                z_ptr,
                Tc_ptr,
                Pc_ptr,
                Dc_ptr,
                ierr_ptr,
                herr_ptr,
                herr_length,
            );
        }

        // Check ierr for errors
        if ierr != 0 {
            // Retrieve the error message using ERRMSGdll
            let mut ierr_errmsg: i32 = 0;

            unsafe {
                bindings::ERRMSGdll(&mut ierr_errmsg as *mut i32, herr_ptr, herr_length);
            }

            // Convert the error message to a Rust String
            let error_message = unsafe {
                CStr::from_ptr(herr_buffer.as_ptr())
                    .to_str()
                    .map_err(|e| RefpropError::Utf8Error(e))?
                    .to_string()
            };

            return Err(RefpropError::CalculationError(error_message));
        }

        // Construct the output struct
        let output = CriticalParameters {
            Tc: Tc_out,
            Pc: Pc_out,
            Dc: Dc_out,
        };

        Ok(output)
    }

    /// Calculates specified single-phase properties using the `ALLPROPS0dll` function.
    ///
    /// This method computes any single-phase property defined in the `i_out` array and returns the values
    /// in the `output` vector. **Note:** This routine should NOT be called for two-phase states!
    ///
    /// The output array is not reset so that several passes can be made to fill in gaps left by previous
    /// calculations (e.g., entries at different `T`, `D`, or `z`). The caller can zero out this array
    /// if desired.
    ///
    /// This routine is designed for advanced users. It optimizes REFPROP property calculations by
    /// converting string identifiers into enumerated integer values, eliminating the overhead of
    /// repeated string comparisons. Since the units are not returned here, refer to the REFPROP
    /// documentation under the molar column.
    ///
    /// **Important:** The `i_out` array should contain enumerated values obtained via the `get_enum` method.
    ///
    /// # Parameters
    ///
    /// - `i_out`: A slice of enumerated integer values identifying the properties to be calculated. Maximum of 200 properties.
    /// - `T`: Temperature [K].
    /// - `D`: Density [mol/L].
    /// - `z`: A slice containing the overall composition (mole fractions). Maximum of 20 components.
    ///
    /// # Returns
    ///
    /// - `Vec<f64>`: A vector containing the calculated property values, in the same order as `i_out`.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `i_out` contains more than 200 elements.
    ///     - `z` contains more than 20 elements.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use refprop_sys::{RefpropFunctionLibrary, GetEnumFlag, RefpropError};
    ///
    /// fn main() -> Result<(), RefpropError> {
    ///     // Define the properties to calculate (e.g., Enthalpy and Entropy)
    ///     let mut i_out = vec![0; 2];
    ///     i_out[0] = RefpropFunctionLibrary::get_enum(GetEnumFlag::AllStrings, "H")?;
    ///     i_out[1] = RefpropFunctionLibrary::get_enum(GetEnumFlag::AllStrings, "S")?;
    ///
    ///     // Define state conditions
    ///     let T = 300.0; // K
    ///     let D = 10.0; // mol/L
    ///     let z = vec![1.0]; // Pure component
    ///
    ///     // Calculate properties
    ///     let output = RefpropFunctionLibrary::all_props0(&i_out, T, D, &z)?;
    ///     println!("Enthalpy: {}, Entropy: {}", output[0], output[1]);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - ALLPROPS0dll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn all_props0(i_out: &[i32], T: f64, D: f64, z: &[f64]) -> Result<Vec<f64>, RefpropError> {
        // Validate input lengths
        if i_out.len() > 200 {
            return Err(RefpropError::InvalidInput(
                "i_out slice length exceeds 200".to_string(),
            ));
        }

        if z.len() > 20 {
            return Err(RefpropError::InvalidInput(
                "z slice length exceeds 20".to_string(),
            ));
        }

        // Acquire the mutex lock to ensure exclusive access
        let _lock = REFPROP_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .map_err(|_| RefpropError::MutexPoisoned)?;

        // Prepare iIn as i32
        let mut i_in = i_out.len() as i32;

        // Prepare iOut buffer with 200 elements, copy i_out into it, pad with 0
        let mut i_out_buffer = [0i32; 200];
        for (i, &val) in i_out.iter().enumerate() {
            i_out_buffer[i] = val;
        }

        // Prepare Output buffer with 200 f64s, initialized to 0
        let mut output_buffer = [0.0f64; 200];

        // Prepare herr buffer with 255 characters
        let mut herr_buffer = vec![0 as libc::c_char; 255];

        // Prepare iFlag (not used, set to 0)
        let mut i_flag: i32 = 0;

        // Prepare z buffer with 20 f64s, copy z into it, pad with 0
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        // Prepare ierr
        let mut ierr: i32 = 0;

        // Call ALLPROPS0dll within unsafe block
        unsafe {
            bindings::ALLPROPS0dll(
                &mut i_in as *mut i32,
                i_out_buffer.as_mut_ptr(),
                &mut i_flag as *mut i32,
                &T as *const f64 as *mut f64,
                &D as *const f64 as *mut f64,
                z_buffer.as_mut_ptr(),
                output_buffer.as_mut_ptr(),
                &mut ierr as *mut i32,
                herr_buffer.as_mut_ptr(),
                255, // herr_length
            );
        }

        // Check ierr for errors
        if ierr != 0 {
            // Retrieve the error message using ERRMSGdll
            let mut ierr_errmsg: i32 = 0;

            unsafe {
                bindings::ERRMSGdll(
                    &mut ierr_errmsg as *mut i32,
                    herr_buffer.as_mut_ptr(),
                    255, // herr_length
                );
            }

            // Convert the error message to a Rust String
            let error_message = unsafe {
                CStr::from_ptr(herr_buffer.as_ptr())
                    .to_str()
                    .map_err(|e| RefpropError::Utf8Error(e))?
                    .to_string()
            };

            return Err(RefpropError::CalculationError(error_message));
        }

        // Collect the first i_in elements of output_buffer
        let output = output_buffer[..i_out.len()].to_vec();

        Ok(output)
    }

    /// Calculates specified single-phase properties using the `ALLPROPS1dll` function.
    ///
    /// This method computes any single-phase property defined in the `h_out` string and returns the
    /// values. **Note:** This routine should NOT be called for two-phase states!
    ///
    /// The `h_out` string can include multiple properties separated by spaces, commas, semicolons, or bars, but only
    /// the first property will be returned.
    /// For example, `"T,P,D,H,E,S"` calculates Temperature, Pressure, Density, Enthalpy, Internal Energy, and Entropy.
    /// To retrieve properties for specific components, use syntax like `"XMOLE(2),XMOLE(3)"`.
    ///
    /// **Important:** The `h_out` string should contain valid REFPROP property identifiers.
    ///
    /// # Parameters
    ///
    /// - `h_out`: A string slice containing the property identifiers to calculate.
    /// - `units`: Specifies the unit system for `T` and `D`. Use `Units::Default` if `T` is in Kelvin and `D` is in mol/dm³.
    /// - `T`: Temperature, with units based on the `units` parameter.
    /// - `D`: Density, with units based on the `units` parameter.
    /// - `z`: A slice containing the overall composition (mole fractions). Maximum of 20 components.
    ///
    /// # Returns
    ///
    /// - `f64`: The calculated property value.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `h_out` contains invalid characters or exceeds 255 characters.
    ///     - `z` contains more than 20 elements.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use refprop_sys::{RefpropFunctionLibrary, Units, RefpropError};
    ///
    /// fn main() -> Result<(), RefpropError> {
    ///     // Define the properties to calculate
    ///     let h_out = "H,S,P";
    ///
    ///     // Define state conditions
    ///     let units = Units::Default; // T in K, D in mol/dm³
    ///     let T = 300.0; // K
    ///     let D = 10.0; // mol/L
    ///     let z = vec![1.0]; // Pure component
    ///
    ///     // Calculate properties
    ///     let output = RefpropFunctionLibrary::all_props1(h_out, units, T, D, &z)?;
    ///     println!("Enthalpy: {}", output);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - ALLPROPS1dll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn all_props1(
        h_out: &str,
        units: Units,
        T: f64,
        D: f64,
        z: &[f64],
    ) -> Result<f64, RefpropError> {
        // Validate input lengths
        if h_out.len() > 255 {
            return Err(RefpropError::InvalidInput(
                "h_out string length exceeds 255 characters".to_string(),
            ));
        }

        if z.len() > 20 {
            return Err(RefpropError::InvalidInput(
                "z slice length exceeds 20".to_string(),
            ));
        }

        // Convert h_out to CString, ensuring no null bytes
        let c_h_out = CString::new(h_out)
            .map_err(|e| RefpropError::InvalidInput(format!("h_out contains null byte: {}", e)))?;

        // Retrieve the iUnits code using the Units enum
        let i_units = units.get_iunits_code()?;

        // Acquire the mutex lock to ensure exclusive access
        let _lock = REFPROP_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .map_err(|_| RefpropError::MutexPoisoned)?;

        // Define buffer sizes as per REFPROP's documentation
        const HOUT_LENGTH: usize = 255;
        const HERR_LENGTH: usize = 255;
        const OUTPUT_LENGTH: usize = 200;

        // Initialize buffers
        let mut c_buffer = [0.0f64; OUTPUT_LENGTH];
        let mut ierr: i32 = 0;
        let mut herr_buffer = vec![0 as libc::c_char; HERR_LENGTH];

        // Convert z slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        // Prepare mutable pointers for FFI
        let h_out_ptr = c_h_out.as_ptr() as *mut libc::c_char;
        let i_units_ptr = &i_units as *const i32 as *mut i32;
        let T_ptr = &T as *const f64 as *mut f64;
        let D_ptr = &D as *const f64 as *mut f64;
        let z_ptr = z_buffer.as_mut_ptr();
        let c_ptr = c_buffer.as_mut_ptr();
        let ierr_ptr = &mut ierr as *mut i32;
        let herr_ptr = herr_buffer.as_mut_ptr();
        let h_out_length = HOUT_LENGTH as i32;
        let herr_length = HERR_LENGTH as i32;

        // Call ALLPROPS1dll within unsafe block
        unsafe {
            for _ in 0..2 {
                bindings::ALLPROPS1dll(
                    h_out_ptr,
                    i_units_ptr,
                    T_ptr,
                    D_ptr,
                    z_ptr,
                    c_ptr,
                    ierr_ptr,
                    herr_ptr,
                    h_out_length,
                    herr_length,
                );
            }
        }

        // Check ierr for errors
        if ierr != 0 {
            // Retrieve the error message using ERRMSGdll
            let mut ierr_errmsg: i32 = 0;

            unsafe {
                bindings::ERRMSGdll(&mut ierr_errmsg as *mut i32, herr_ptr, herr_length);
            }

            // Convert the error message to a Rust String
            let error_message = unsafe {
                CStr::from_ptr(herr_buffer.as_ptr())
                    .to_str()
                    .map_err(|e| RefpropError::Utf8Error(e))?
                    .to_string()
            };

            return Err(RefpropError::CalculationError(error_message));
        }

        // Collect the output values
        // According to documentation, properties are returned in the order specified by h_out
        // For simplicity, we'll collect all 200 outputs, but in practice, only relevant ones are filled
        // Here, we'll filter out -9999970 which indicates errors or no input
        let output: f64 = c_buffer[0];
        if output == -999970.0 {
            return Err(RefpropError::CalculationError(
                "REFPROP unable to calculate output.".into(),
            ));
        }

        Ok(output)
    }
}
