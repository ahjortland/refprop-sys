use crate::{
    bindings,
    utils::{acquire_lock, validate_composition},
    RefpropError, RefpropFunctionLibrary,
};

impl RefpropFunctionLibrary {
    /// Converts a composition from mass fractions to mole fractions using the `XMOLEdll` function.
    ///
    /// This method takes a composition given in mass fractions (`xkg`) and converts it to mole fractions (`xmol`),
    /// also calculating the molar mass of the mixture (`wmix`).
    ///
    /// **Note:** Ensure that the mass fractions sum to 1 within a reasonable tolerance before calling this function.
    ///
    /// # Parameters
    ///
    /// - `xkg`: A slice containing the composition in mass fractions. Maximum of 20 components.
    ///
    /// # Returns
    ///
    /// - `XmoleOutput`: A struct containing the converted mole fractions and the molar mass of the mixture.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::InvalidInput` if:
    ///     - `xkg` contains more than 20 elements.
    ///     - The sum of mass fractions does not equal 1 within a specified tolerance.
    /// - Returns `RefpropError::CalculationError` if REFPROP encounters an error during calculation.
    /// - Returns `RefpropError::Utf8Error` if error messages cannot be converted to UTF-8.
    /// - Returns `RefpropError::MutexPoisoned` if the REFPROP mutex is poisoned.
    ///
    /// # References
    ///
    /// - [REFPROP Documentation - XMOLEdll](https://pages.nist.gov/RefProp/documentation.html)
    pub fn convert_to_mole_fractions(
        mass_fractions: &[f64],
    ) -> Result<(Vec<f64>, f64), RefpropError> {
        // Validate composition slice length
        validate_composition(mass_fractions)?;

        // Acquire the mutex lock to ensure exclusive access
        let _guard = acquire_lock()?;

        // Convert 'xkg' slice to a fixed-size array with padding
        let mut mass_fraction_buffer = [0.0f64; 20];
        for (i, &val) in mass_fractions.iter().enumerate() {
            mass_fraction_buffer[i] = val;
        }

        // Initialize output buffers
        let mut mole_fractions = [0.0f64; 20];
        let mut molar_mass: f64 = 0.0;

        // Prepare mutable pointers for FFI
        let mass_fractions_ptr = mass_fraction_buffer.as_mut_ptr();
        let mole_fractions_ptr = mole_fractions.as_mut_ptr();
        let molar_mass_ptr = &mut molar_mass as *mut f64;

        // Call XMOLEdll within unsafe block
        unsafe {
            bindings::XMOLEdll(mass_fractions_ptr, mole_fractions_ptr, molar_mass_ptr);
        }

        // Since XMOLEdll does not provide an error flag, we rely on the molar mass being positive
        if molar_mass <= 0.0 {
            return Err(RefpropError::CalculationError(format!(
                "Invalid molar mass calculated: {}",
                molar_mass
            )));
        }

        // Convert the mole fractions array to a Vec<f64>
        let mole_fractions = mole_fractions
            .into_iter()
            .take(mass_fractions.len())
            .collect::<Vec<f64>>();

        Ok((mole_fractions, molar_mass))
    }
}
