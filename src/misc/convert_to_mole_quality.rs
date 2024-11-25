use crate::{
    bindings,
    utils::{acquire_lock, check_refprop_error, validate_composition},
    RefpropError, RefpropFunctionLibrary,
};

use super::QualityOutput;

impl RefpropFunctionLibrary {
    /// Converts quality and composition on a mass basis to a molar basis using the `QMOLEdll` function.
    ///
    /// This method takes the mass quality (`qkg`), composition of liquid phase (`xlkg`), and composition of vapor phase (`xvkg`)
    /// to compute the molar quality (`qmol`), mole compositions (`xl` and `xv`), and molar masses of liquid and vapor phases (`wliq` and `wvap`).
    ///
    /// **Note:** Ensure that the mass fractions in `xlkg` and `xvkg` sum to 1 within a reasonable tolerance before calling this function.
    ///
    /// # Parameters
    ///
    /// - `qkg`: Quality on mass basis (mass of vapor/total mass). `qkg = 0` indicates saturated liquid, `qkg = 1` indicates saturated vapor. `0 < qkg < 1` indicates a two-phase state. Values outside `[0, 1]` are invalid.
    /// - `xlkg`: A slice containing the composition of the liquid phase in mass fractions. Maximum of 20 components.
    /// - `xvkg`: A slice containing the composition of the vapor phase in mass fractions. Maximum of 20 components.
    ///
    /// # Returns
    ///
    /// - `QmoleOutput`: A struct containing the molar quality, mole compositions, and molar masses of the liquid and vapor phases.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `xlkg` or `xvkg` contains more than 20 elements.
    ///     - The sum of mass fractions in `xlkg` or `xvkg` does not equal 1 within a specified tolerance.
    ///     - `qkg` is outside the range `[0, 1]`.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - QMOLEdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn convert_to_mole_quality(
        mass_quality: f64,
        mass_fractions_liquid: &[f64],
        mass_fractions_vapor: &[f64],
    ) -> Result<QualityOutput, RefpropError> {
        // Validate quality
        if mass_quality < 0.0 || mass_quality > 1.0 {
            return Err(RefpropError::InvalidInput(format!(
                "Quality 'qkg' must be between 0 and 1. Provided: {}",
                mass_quality
            )));
        }

        // Validate composition slices length
        validate_composition(mass_fractions_liquid)?;
        validate_composition(mass_fractions_vapor)?;

        // Acquire the mutex lock to ensure exclusive access
        let guard = acquire_lock()?;

        // Convert 'xlkg' and 'xvkg' slices to fixed-size arrays with padding
        let mut mass_fractions_liquid_buffer = [0.0f64; 20];
        for (i, &val) in mass_fractions_liquid.iter().enumerate() {
            mass_fractions_liquid_buffer[i] = val;
        }
        let mut mass_fractions_vapor_buffer = [0.0f64; 20];
        for (i, &val) in mass_fractions_vapor.iter().enumerate() {
            mass_fractions_vapor_buffer[i] = val;
        }

        // Initialize output buffers
        let mut mole_quality: f64 = 0.0;
        let mut mole_fractions_liquid = [0.0f64; 20];
        let mut mole_fractions_vapor = [0.0f64; 20];
        let mut molar_mass_liquid: f64 = 0.0;
        let mut molar_mass_vapor: f64 = 0.0;
        let mut ierr: i32 = 0;
        let mut herr_buffer = vec![0 as libc::c_char; 255];

        // Prepare mutable pointers for FFI
        let mass_quality_ptr = &mass_quality as *const f64 as *mut f64;
        let mass_fractions_liquid_ptr = mass_fractions_liquid_buffer.as_mut_ptr();
        let mass_fractions_vapor_ptr = mass_fractions_vapor_buffer.as_mut_ptr();
        let mole_quality_ptr = &mut mole_quality as *mut f64;
        let mole_fractions_liquid_ptr = mole_fractions_liquid.as_mut_ptr();
        let mole_fractions_vapor_ptr = mole_fractions_vapor.as_mut_ptr();
        let molar_mass_liquid_ptr = &mut molar_mass_liquid as *mut f64;
        let molar_mass_vapor_ptr = &mut molar_mass_vapor as *mut f64;
        let ierr_ptr = &mut ierr as *mut i32;
        let herr_ptr = herr_buffer.as_mut_ptr();
        let herr_length = 255 as i32;

        // Call QMOLEdll within unsafe block
        unsafe {
            bindings::QMOLEdll(
                mass_quality_ptr,
                mass_fractions_liquid_ptr,
                mass_fractions_vapor_ptr,
                mole_quality_ptr,
                mole_fractions_liquid_ptr,
                mole_fractions_vapor_ptr,
                molar_mass_liquid_ptr,
                molar_mass_vapor_ptr,
                ierr_ptr,
                herr_ptr,
                herr_length,
            );
        }

        // Check ierr for errors
        check_refprop_error(&guard, ierr, herr_ptr, herr_length)?;

        // Convert the mole fractions arrays to Vec<f64>
        let mole_fractions_liquid = mole_fractions_liquid
            .into_iter()
            .take(mass_fractions_liquid.len())
            .collect::<Vec<f64>>();
        let mole_fractions_vapor = mole_fractions_vapor
            .into_iter()
            .take(mass_fractions_vapor.len())
            .collect::<Vec<f64>>();

        // Construct the output struct
        let output = QualityOutput {
            quality: mole_quality,
            liq_composition: mole_fractions_liquid,
            vap_composition: mole_fractions_vapor,
            liq_molar_mass: molar_mass_liquid,
            vap_molar_mass: molar_mass_vapor,
        };

        Ok(output)
    }
}
