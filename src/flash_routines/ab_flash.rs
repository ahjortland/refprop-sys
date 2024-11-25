use std::ffi::CString;

use crate::{
    bindings,
    utils::{acquire_lock, check_refprop_error, validate_composition},
    Basis, KrKqFlag, Phase, RefpropError, RefpropFunctionLibrary, CP_UNDEFINED, CV_UNDEFINED,
};

use super::FlashOutput;

impl RefpropFunctionLibrary {
    /// Performs a general flash calculation using the `ABFLSHdll` function.
    ///
    /// This method calculates various thermodynamic properties based on the provided input properties.
    /// It handles both single-phase and two-phase states.
    ///
    /// **Note:** This routine should NOT be called for two-phase states if not intended, as it can return undefined properties.
    ///
    /// # Parameters
    ///
    /// - `ab`: A string slice composed of two letters indicating the input properties (e.g., `"PH"`, `"TD"`).
    ///         Valid characters:
    ///         - `T` - Temperature [K]
    ///         - `P` - Pressure [kPa]
    ///         - `D` - Density [mol/L or kg/m³]
    ///         - `E` - Internal energy [J/mol or kJ/kg]
    ///         - `H` - Enthalpy [J/mol or kJ/kg]
    ///         - `S` - Entropy [J/mol-K or kJ/kg-K]
    ///         - `Q` - Quality [mol/mol or kJ/kg]
    ///
    /// - `a`: The value of the first property specified in `ab`.
    /// - `b`: The value of the second property specified in `ab`.
    /// - `z`: A slice containing the overall composition (mole fractions). Maximum of 20 components.
    ///         For saturation properties (`"TQ"` or `"PQ"`), send `b = -99` for melting line states and `b = -98` for sublimation line states.
    /// - `flags`: An `AbfleshFlags` struct specifying the combined flags for the calculation.
    ///
    /// # Returns
    ///
    /// - `AbfleshOutput`: A struct containing the calculated thermodynamic properties.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `ab` does not contain exactly two valid characters.
    ///     - `ab` contains invalid characters.
    ///     - `z` contains more than 20 elements.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use refprop_sys::{RefpropFunctionLibrary, Basis, Phase, KrKqFlag, RefpropError};
    ///
    /// fn main() -> Result<(), RefpropError> {
    ///     let _ = RefpropFunctionLibrary::set_path(None);
    ///     let _ = RefpropFunctionLibrary::set_fluids("R32");
    ///
    ///     // Define input properties: Pressure (P) and Enthalpy (H)
    ///     let ab = "PH";
    ///     let a = 101.325; // kPa
    ///     let b = 500.0;   // J/mol
    ///
    ///     // Define composition (pure component)
    ///     let z = vec![1.0];
    ///
    ///     // Define flags: Molar basis, Unknown phase, Default kr/kq
    ///     let imass = Basis::Molar;
    ///     let kph = Phase::Unknown;
    ///     let krkq = KrKqFlag::Default;
    ///
    ///     // Perform flash calculation
    ///     let output = RefpropFunctionLibrary::ab_flash(ab, a, b, &z, imass, kph, krkq)?;
    ///
    ///     // Access the calculated properties
    ///     println!("Temperature: {} K", output.T);
    ///     println!("Pressure: {} kPa", output.P);
    ///     println!("Density: {}", output.D);
    ///     // ... access other properties as needed
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - ABFLSHdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn ab_flash(
        ab: &str,
        a: f64,
        b: f64,
        z: &[f64],
        imass_flag: Basis,
        kph_flag: Phase,
        krkq_flag: KrKqFlag,
    ) -> Result<FlashOutput, RefpropError> {
        // Validate 'ab' string
        if ab.len() != 2 {
            return Err(RefpropError::InvalidInput(
                "ab string must be exactly two characters long".to_string(),
            ));
        }

        // Ensure 'ab' contains only valid characters
        let valid_chars = ['T', 'P', 'D', 'E', 'H', 'S', 'Q'];
        let ab_upper = ab.to_uppercase();
        for ch in ab_upper.chars() {
            if !valid_chars.contains(&ch) {
                return Err(RefpropError::InvalidInput(format!(
                    "Invalid character '{}' in ab string",
                    ch
                )));
            }
        }

        // Validate composition slice length
        validate_composition(z)?;

        // Acquire the mutex lock to ensure exclusive access
        let lock = acquire_lock()?;

        // Convert 'ab' to CString, ensuring no null bytes
        let c_ab = CString::new(ab_upper.as_str())
            .map_err(|e| RefpropError::InvalidInput(format!("ab contains null byte: {}", e)))?;

        // Retrieve the combined iFlag integer
        let iflag = imass_flag as i32 + 10 * kph_flag as i32 + 100 * krkq_flag as i32;

        // Define buffer sizes as per REFPROP's documentation
        const AB_LENGTH: usize = 2;
        const HERR_LENGTH: usize = 255;

        // Initialize output buffers
        let mut T: f64 = 0.0;
        let mut P: f64 = 0.0;
        let mut D: f64 = 0.0;
        let mut Dl: f64 = 0.0;
        let mut Dv: f64 = 0.0;
        let mut x = [0.0f64; 20];
        let mut y = [0.0f64; 20];
        let mut q: f64 = 0.0;
        let mut e: f64 = 0.0;
        let mut h: f64 = 0.0;
        let mut s: f64 = 0.0;
        let mut Cv: f64 = CV_UNDEFINED;
        let mut Cp: f64 = CP_UNDEFINED;
        let mut w: f64 = 0.0;
        let mut ierr: i32 = 0;
        let mut herr_buffer = vec![0 as libc::c_char; HERR_LENGTH];

        // Convert z slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        // Prepare mutable pointers for FFI
        let ab_ptr = c_ab.as_ptr() as *mut libc::c_char;
        let a_ptr = &a as *const f64 as *mut f64;
        let b_ptr = &b as *const f64 as *mut f64;
        let z_ptr = z_buffer.as_mut_ptr();
        let iflag_ptr = &iflag as *const i32 as *mut i32;
        let T_ptr = &mut T as *mut f64;
        let P_ptr = &mut P as *mut f64;
        let D_ptr = &mut D as *mut f64;
        let Dl_ptr = &mut Dl as *mut f64;
        let Dv_ptr = &mut Dv as *mut f64;
        let x_ptr = x.as_mut_ptr();
        let y_ptr = y.as_mut_ptr();
        let q_ptr = &mut q as *mut f64;
        let e_ptr = &mut e as *mut f64;
        let h_ptr = &mut h as *mut f64;
        let s_ptr = &mut s as *mut f64;
        let Cv_ptr = &mut Cv as *mut f64;
        let Cp_ptr = &mut Cp as *mut f64;
        let w_ptr = &mut w as *mut f64;
        let ierr_ptr = &mut ierr as *mut i32;
        let herr_ptr = herr_buffer.as_mut_ptr();
        let ab_length = AB_LENGTH as i32;
        let herr_length = HERR_LENGTH as i32;

        // Call ABFLSHdll within unsafe block
        unsafe {
            bindings::ABFLSHdll(
                ab_ptr,
                a_ptr,
                b_ptr,
                z_ptr,
                iflag_ptr,
                T_ptr,
                P_ptr,
                D_ptr,
                Dl_ptr,
                Dv_ptr,
                x_ptr,
                y_ptr,
                q_ptr,
                e_ptr,
                h_ptr,
                s_ptr,
                Cv_ptr,
                Cp_ptr,
                w_ptr,
                ierr_ptr,
                herr_ptr,
                ab_length,
                herr_length,
            );
        }

        // Check ierr for errors
        check_refprop_error(&lock, ierr, herr_ptr, herr_length)?;

        // Prepare composition vectors
        let x = x.into_iter().take(z.len()).collect::<Vec<f64>>();
        let y = y.into_iter().take(z.len()).collect::<Vec<f64>>();

        // Handle optional properties (Cv and Cp)
        let Cv = if Cv == CV_UNDEFINED || Cv == CP_UNDEFINED {
            None
        } else {
            Some(Cv)
        };
        let Cp = if Cp == CP_UNDEFINED || Cp == CV_UNDEFINED {
            None
        } else {
            Some(Cp)
        };

        // Construct the output struct
        let output = FlashOutput {
            T,
            P,
            D,
            Dl,
            Dv,
            x,
            y,
            q,
            e,
            h,
            s,
            Cv,
            Cp,
            w,
        };

        Ok(output)
    }
}
