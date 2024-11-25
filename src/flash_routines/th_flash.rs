use crate::{
    bindings,
    flash_routines::FlashOutput,
    utils::{acquire_lock, check_refprop_error, validate_composition},
    RefpropError, RefpropFunctionLibrary, CP_UNDEFINED, CV_UNDEFINED,
};

impl RefpropFunctionLibrary {
    /// Performs a flash calculation given temperature, enthalpy, and bulk composition using the `THFLSHdll` function.
    ///
    /// This method computes thermodynamic properties based on the provided inputs. It can handle both single-phase and two-phase states.
    ///
    /// **Note:** If multiple solutions exist (e.g., two liquid phases), set `kr = 2` to obtain a single-phase state.
    ///
    /// # Parameters
    ///
    /// - `T`: Temperature [K]
    /// - `h`: Enthalpy [J/mol]
    /// - `z`: Bulk Composition (slice of mole fractions). Maximum of 20 components.
    /// - `kr`: Integer flag. Typically set to `1`. Set to `2` to obtain a single-phase state if multiple solutions exist.
    ///
    /// # Returns
    ///
    /// - `FlashOutput`: A struct containing the calculated properties.
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
    /// - [REFPROP Documentation - THFLSHdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn th_flash(T: f64, h: f64, z: &[f64], kr: i32) -> Result<FlashOutput, RefpropError> {
        // Validate composition slice length
        validate_composition(z)?;

        // Validate kr (assuming valid values are 1 or 2 based on documentation)
        if kr != 1 && kr != 2 {
            return Err(RefpropError::InvalidInput(
                "Parameter 'kr' must be either 1 or 2.".to_string(),
            ));
        }

        // Acquire the mutex lock to ensure exclusive access
        let _lock = acquire_lock()?;

        // Convert 'z' slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        let mut T = T;
        let mut h = h;

        // Initialize output buffers
        let mut P: f64 = 0.0;
        let mut D: f64 = 0.0;
        let mut Dl: f64 = 0.0;
        let mut Dv: f64 = 0.0;
        let mut x = [0.0f64; 20];
        let mut y = [0.0f64; 20];
        let mut q: f64 = 0.0;
        let mut e: f64 = 0.0;
        let mut s: f64 = 0.0;
        let mut Cv: f64 = CV_UNDEFINED; // Sentinel value indicating undefined
        let mut Cp: f64 = CP_UNDEFINED; // Sentinel value indicating undefined
        let mut w: f64 = 0.0;
        let mut ierr: i32 = 0;
        let mut herr_buffer = vec![0 as libc::c_char; 255];

        // Prepare mutable pointers for FFI
        let T_ptr = &mut T as *mut f64;
        let h_ptr = &mut h as *mut f64;
        let z_ptr = z_buffer.as_mut_ptr();
        let kr_ptr = &kr as *const i32 as *mut i32;
        let P_ptr = &mut P as *mut f64;
        let D_ptr = &mut D as *mut f64;
        let Dl_ptr = &mut Dl as *mut f64;
        let Dv_ptr = &mut Dv as *mut f64;
        let x_ptr = x.as_mut_ptr();
        let y_ptr = y.as_mut_ptr();
        let q_ptr = &mut q as *mut f64;
        let e_ptr = &mut e as *mut f64;
        let s_ptr = &mut s as *mut f64;
        let Cv_ptr = &mut Cv as *mut f64;
        let Cp_ptr = &mut Cp as *mut f64;
        let w_ptr = &mut w as *mut f64;
        let ierr_ptr = &mut ierr as *mut i32;
        let herr_ptr = herr_buffer.as_mut_ptr();
        let herr_length = 255 as i32;

        // Call THFLSHdll within unsafe block
        unsafe {
            bindings::THFLSHdll(
                T_ptr,
                h_ptr,
                z_ptr,
                kr_ptr,
                P_ptr,
                D_ptr,
                Dl_ptr,
                Dv_ptr,
                x_ptr,
                y_ptr,
                q_ptr,
                e_ptr,
                s_ptr,
                Cv_ptr,
                Cp_ptr,
                w_ptr,
                ierr_ptr,
                herr_ptr,
                herr_length,
            );
        }

        // Check ierr for errors
        check_refprop_error(&_lock, ierr, herr_ptr, herr_length)?;

        // Handle optional properties (Cv and Cp)
        let Cv = if Cv == CV_UNDEFINED { None } else { Some(Cv) };
        let Cp = if Cp == CP_UNDEFINED { None } else { Some(Cp) };

        // Convert the composition arrays to Vec<f64>
        let x = x.into_iter().take(z.len()).collect::<Vec<f64>>();
        let y = y.into_iter().take(z.len()).collect::<Vec<f64>>();

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
