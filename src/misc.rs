mod calc_molar_mass;
mod convert_to_mass_fractions;
mod convert_to_mass_quality;
mod convert_to_mole_fractions;
mod convert_to_mole_quality;
pub(crate) mod get_enum;
mod transport;

/// Represents the output of the `qmole` method.
#[derive(Debug, Clone)]
pub struct QualityOutput {
    /// Quality on molar or mass basis (moles/mass of vapor/total moles/mass).
    pub quality: f64,
    /// Composition of the liquid phase in mole fractions (xmol) [unitless].
    pub liq_composition: Vec<f64>,
    /// Composition of the vapor phase in mole fractions (xmol) [unitless].
    pub vap_composition: Vec<f64>,
    /// Molar mass of the liquid phase [g/mol].
    pub liq_molar_mass: f64,
    /// Molar mass of the vapor phase [g/mol].
    pub vap_molar_mass: f64,
}

#[derive(Debug, Clone)]
pub struct TransportOutput {
    /// Dynamic viscosity [uPa-s].
    pub eta: f64,
    /// Thermal conductivity [W/(m-K)].
    pub tcx: f64,
}
