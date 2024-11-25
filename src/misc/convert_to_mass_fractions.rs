use crate::{
    bindings,
    utils::{acquire_lock, validate_composition},
    RefpropError, RefpropFunctionLibrary,
};

impl RefpropFunctionLibrary {
    /// Converts a composition from mole fractions to mass fractions using the `XMASSdll` function.
    ///
    /// This method takes a composition given in mole fractions (`xmol`) and converts it to mass fractions (`xkg`),
    /// also calculating the molar mass of the mixture (`wmix`).
    ///
    /// **Note:** Ensure that the mole fractions sum to 1 within a reasonable tolerance before calling this function.
    ///
    /// # Parameters
    ///
    /// - `xmol`: A slice containing the composition in mole fractions. Maximum of 20 components.
    ///
    /// # Returns
    ///
    /// - `XmassOutput`: A struct containing the converted mass fractions and the molar mass of the mixture.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `xmol` contains more than 20 elements.
    ///     - The sum of mole fractions does not equal 1 within a specified tolerance.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - XMASSdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn convert_to_mass_fractions(
        mole_fractions: &[f64],
    ) -> Result<(Vec<f64>, f64), RefpropError> {
        // Validate composition slice length
        validate_composition(mole_fractions)?;

        // Acquire the mutex lock to ensure exclusive access
        let _guard = acquire_lock()?;

        // Convert 'z' slice to a fixed-size array with padding
        let mut xmol_buffer = [0.0f64; 20];
        for (i, &val) in mole_fractions.iter().enumerate() {
            xmol_buffer[i] = val;
        }

        // Initialize output buffers
        let mut mass_fractions = [0.0f64; 20];
        let mut molar_mass: f64 = 0.0;

        // Prepare mutable pointers for FFI
        let mole_fractions_ptr = xmol_buffer.as_mut_ptr();
        let mass_fractions_ptr = mass_fractions.as_mut_ptr();
        let molar_mass_ptr = &mut molar_mass as *mut f64;

        // Call XMASSdll within unsafe block
        unsafe {
            bindings::XMASSdll(mole_fractions_ptr, mass_fractions_ptr, molar_mass_ptr);
        }

        // Since XMASSdll does not provide an error flag, we rely on the molar mass being positive
        if molar_mass <= 0.0 {
            return Err(RefpropError::CalculationError(format!(
                "Invalid molar mass calculated: {}",
                molar_mass
            )));
        }

        // Convert the mass fractions array to a Vec<f64>
        let mass_fractions = mass_fractions
            .into_iter()
            .take(mole_fractions.len())
            .collect::<Vec<f64>>();

        Ok((mass_fractions, molar_mass))
    }
}
