mod ab_flash;
mod de_flash;
mod dh_flash;
mod ds_flash;
mod hs_flash;
mod pd_flash;
mod pe_flash;
mod ph_flash;
mod pq_flash;
mod ps_flash;
mod td_flash;
mod te_flash;
mod th_flash;
mod tp_flash;
mod tq_flash;
mod ts_flash;

// Represents the output properties from the flash routines.
#[derive(Debug, Clone)]
pub struct FlashOutput {
    /// Temperature [K]
    pub T: f64,
    /// Pressure [kPa]
    pub P: f64,
    /// Density [mol/L or kg/m³]
    pub D: f64,
    /// Molar density of the liquid phase [mol/L or kg/m³]
    pub Dl: f64,
    /// Molar density of the vapor phase [mol/L or kg/m³]
    pub Dv: f64,
    /// Composition of the liquid phase (mole or mass fractions)
    pub x: Vec<f64>,
    /// Composition of the vapor phase (mole or mass fractions)
    pub y: Vec<f64>,
    /// Vapor quality on a MOLAR basis (moles of vapor/total moles)
    pub q: f64,
    /// Overall internal energy [J/mol or kJ/kg]
    pub e: f64,
    /// Overall enthalpy [J/mol or kJ/kg]
    pub h: f64,
    /// Overall entropy [J/mol-K or kJ/kg-K]
    pub s: f64,
    /// Isochoric (constant D) heat capacity [J/mol-K or kJ/kg-K]
    pub Cv: Option<f64>, // Not defined for 2-phase states
    /// Isobaric (constant P) heat capacity [J/mol-K or kJ/kg-K]
    pub Cp: Option<f64>, // Not defined for 2-phase states
    /// Speed of sound [m/s]
    pub w: f64,
}
