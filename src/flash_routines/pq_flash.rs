use crate::{
    bindings,
    flash_routines::FlashOutput,
    utils::{acquire_lock, check_refprop_error, validate_composition},
    Basis, KrKqFlag, Phase, RefpropError, RefpropFunctionLibrary, CP_UNDEFINED, CV_UNDEFINED,
};

impl RefpropFunctionLibrary {
    /// Performs a flash calculation given pressure, quality, and bulk composition using the `PQFLSHdll` function.
    ///
    /// **Note:** This routine accepts saturation or two-phase states as inputs. Ensure that the fluid is set prior to calling this function.
    ///
    /// # Parameters
    ///
    /// - `P`: Pressure [kPa]
    /// - `q`: Vapor quality [mol/mol]
    /// - `z`: A slice containing the bulk composition (mole fractions). Maximum of 20 components.
    /// - `kq`: A `KqFlag` enum specifying the behavior of the flash calculation.
    ///
    /// # Returns
    ///
    /// - `FlashOutput`: A struct containing the calculated thermodynamic properties.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `z` contains more than 20 elements.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - PQFLSHdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn pq_flash(
        P: f64,
        q: f64,
        z: &[f64],
        imass_flag: Basis,
        kph_flag: Phase,
        krkq_flag: KrKqFlag,
    ) -> Result<FlashOutput, RefpropError> {
        // Validate composition slice length
        validate_composition(z)?;

        // Acquire the mutex lock to ensure exclusive access
        let _lock = acquire_lock()?;

        // Convert 'z' slice to a fixed-size array with padding
        let mut z_buffer = [0.0f64; 20];
        for (i, &val) in z.iter().enumerate() {
            z_buffer[i] = val;
        }

        // Retrieve the combined iFlag integer
        let iflag = imass_flag as i32 + 10 * kph_flag as i32 + 100 * krkq_flag as i32;

        // Define buffer sizes as per REFPROP's documentation
        const HERR_LENGTH: usize = 255;

        let mut P = P;
        let mut q = q;

        // Initialize output buffers
        let mut T: f64 = 0.0;
        let mut D: f64 = 0.0;
        let mut Dl: f64 = 0.0;
        let mut Dv: f64 = 0.0;
        let mut x = [0.0f64; 20];
        let mut y = [0.0f64; 20];
        let mut e: f64 = 0.0;
        let mut h: f64 = 0.0;
        let mut s: f64 = 0.0;
        let mut Cv: f64 = CV_UNDEFINED;
        let mut Cp: f64 = CP_UNDEFINED;
        let mut w: f64 = 0.0;
        let mut ierr: i32 = 0;
        let mut herr_buffer = vec![0 as libc::c_char; HERR_LENGTH];

        // Prepare mutable pointers for FFI
        let T_ptr = &mut T as *mut f64;
        let q_ptr = &mut q as *mut f64;
        let z_ptr = z_buffer.as_mut_ptr();
        let kq_int = iflag;
        let kq_ptr = &kq_int as *const i32 as *mut i32;
        let P_ptr = &mut P as *mut f64;
        let D_ptr = &mut D as *mut f64;
        let Dl_ptr = &mut Dl as *mut f64;
        let Dv_ptr = &mut Dv as *mut f64;
        let x_ptr = x.as_mut_ptr();
        let y_ptr = y.as_mut_ptr();
        let e_ptr = &mut e as *mut f64;
        let h_ptr = &mut h as *mut f64;
        let s_ptr = &mut s as *mut f64;
        let Cv_ptr = &mut Cv as *mut f64;
        let Cp_ptr = &mut Cp as *mut f64;
        let w_ptr = &mut w as *mut f64;
        let ierr_ptr = &mut ierr as *mut i32;
        let herr_ptr = herr_buffer.as_mut_ptr();
        let herr_length = HERR_LENGTH as i32;

        // Call TQFLSHdll within unsafe block
        unsafe {
            bindings::PQFLSHdll(
                P_ptr,
                q_ptr,
                z_ptr,
                kq_ptr,
                T_ptr,
                D_ptr,
                Dl_ptr,
                Dv_ptr,
                x_ptr,
                y_ptr,
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

        // Prepare composition vectors
        let x = x.into_iter().take(z.len()).collect::<Vec<f64>>();
        let y = y.into_iter().take(z.len()).collect::<Vec<f64>>();

        // Handle optional properties (Cv and Cp)
        let Cv = if Cv == CV_UNDEFINED { None } else { Some(Cv) };
        let Cp = if Cp == CP_UNDEFINED { None } else { Some(Cp) };

        // Construct the output struct
        let output = FlashOutput {
            T,
            P,
            D,
            Dl,
            Dv,
            x,
            y,
            e,
            h,
            s,
            Cv,
            Cp,
            w,
            q,
        };

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pq_flash() -> Result<(), RefpropError> {
        // Define composition (binary mixture)
        let _ = RefpropFunctionLibrary::set_path(None);
        let z = RefpropFunctionLibrary::set_mixture("R454B")?;
        // z.truncate(2);

        let P = 1300.0;
        let q = 0.9;
        let result = RefpropFunctionLibrary::pq_flash(
            P,
            q,
            &z,
            Basis::Molar,
            Phase::TwoPhase,
            KrKqFlag::Default,
        )?;

        println!("{:?}", result);

        Ok(())
    }
}
