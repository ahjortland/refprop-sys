use crate::{
    bindings,
    flash_routines::FlashOutput,
    utils::{acquire_lock, check_refprop_error, validate_composition},
    RefpropError, RefpropFunctionLibrary, CP_UNDEFINED, CV_UNDEFINED,
};

impl RefpropFunctionLibrary {
    /// Performs a flash calculation given temperature, bulk density, and bulk composition using the `TDFLSHdll` function.
    ///
    /// This method computes thermodynamic properties based on the provided inputs. It can handle both single-phase and two-phase states.
    ///
    /// **Note:** For single-phase calculations, the subroutine `THERM` is much faster.
    ///
    /// # Parameters
    ///
    /// - `T`: Temperature [K]
    /// - `D`: Density [mol/L]
    /// - `z`: Bulk Composition (slice of mole fractions). Maximum of 20 components.
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
    /// - [REFPROP Documentation - TDFLSHdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn td_flash(T: f64, D: f64, z: &[f64]) -> Result<FlashOutput, RefpropError> {
        // Validate composition slice length
        validate_composition(z)?;

        // Acquire the mutex lock to ensure exclusive access
        let _lock = acquire_lock()?;

        // Convert 'z' slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        let mut T = T;
        let mut D = D;

        // Initialize output buffers
        let mut P: f64 = 0.0;
        let mut Dl: f64 = 0.0;
        let mut Dv: f64 = 0.0;
        let mut x = [0.0f64; 20];
        let mut y = [0.0f64; 20];
        let mut q: f64 = 0.0;
        let mut e: f64 = 0.0;
        let mut h: f64 = 0.0;
        let mut s: f64 = 0.0;
        let mut Cv: f64 = CV_UNDEFINED; // Sentinel value
        let mut Cp: f64 = CP_UNDEFINED; // Sentinel value
        let mut w: f64 = 0.0;
        let mut ierr: i32 = 0;
        let mut herr_buffer = vec![0 as libc::c_char; 255];

        // Prepare mutable pointers for FFI
        let T_ptr = &mut T as *mut f64;
        let D_ptr = &mut D as *mut f64;
        let z_ptr = z_buffer.as_mut_ptr();
        let P_ptr = &mut P as *mut f64;
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
        let herr_length = 255 as i32;

        // Call TDFLSHdll within unsafe block
        unsafe {
            bindings::TDFLSHdll(
                T_ptr,
                D_ptr,
                z_ptr,
                P_ptr,
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
                herr_length,
            );
        }

        // Check ierr for errors
        check_refprop_error(&_lock, ierr, herr_ptr, herr_length)?;

        // Handle optional properties (Cv and Cp)
        let Cv = if Cv == CV_UNDEFINED { None } else { Some(Cv) };
        let Cp = if Cp == CP_UNDEFINED { None } else { Some(Cp) };

        // Convert the composition arrays to Vec<f64>
        let x = x.iter().take(z.len()).cloned().collect::<Vec<f64>>();
        let y = y.iter().take(z.len()).cloned().collect::<Vec<f64>>();

        // Construct the output struct
        let output = FlashOutput {
            T,
            D,
            P,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_td_flash() -> Result<(), RefpropError> {
        // Define composition (binary mixture)
        let _ = RefpropFunctionLibrary::set_path(None);
        let z = RefpropFunctionLibrary::set_mixture("R454B")?;
        // z.truncate(2);

        let T = 300.0;
        let D = 0.5;
        let result = RefpropFunctionLibrary::td_flash(T, D, &z)?;

        println!("{:?}", result);

        Ok(())
    }
}