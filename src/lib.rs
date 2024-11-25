#![allow(non_snake_case)]
pub(crate) mod bindings;
pub mod errors;
mod flash_routines;
mod misc;
mod setup;
pub(crate) mod utils;

use std::sync::{Mutex, OnceLock};

pub use errors::RefpropError;
pub use misc::get_enum::GetEnumFlag;

pub(crate) static REFPROP_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) const CV_UNDEFINED: f64 = -9999990.0;
pub(crate) const CP_UNDEFINED: f64 = -9999980.0;

/// Represents the unit systems for temperature and density inputs.
#[derive(Debug, Clone)]
pub enum Units {
    /// Default units: Temperature in Kelvin (K) and Density in mol/dm³.
    Default,
    /// Molar SI units.
    MolarSI,
    /// Mass SI units.
    MassSI,
    /// SI units with Celsius temperature.
    SIWithC,
    /// Molar Base SI units.
    MolarBaseSI,
    /// Mass Base SI units.
    MassBaseSI,
    /// English units.
    English,
    /// Molar English units.
    MolarEnglish,
    /// MKS units.
    MKS,
    /// CGS units.
    CGS,
    /// Mixed units.
    Mixed,
    /// MEUNITS.
    MEUnits,
    /// User-defined units.
    User,
    /// Custom unit systems (future or undefined).
    Custom(String),
}

impl Units {
    /// Retrieves the integer code corresponding to the unit system by calling `get_enum`.
    ///
    /// This method maps each `Units` variant to its corresponding string identifier as defined in REFPROP
    /// and uses the `get_enum` method to obtain the enumerated integer value.
    ///
    /// # Errors
    ///
    /// - Returns `RefpropError::CalculationError` if the unit system string is invalid or REFPROP encounters an error.
    pub fn get_iunits_code(&self) -> Result<i32, RefpropError> {
        let enum_str = match self {
            Units::Default => "DEFAULT",
            Units::MolarSI => "MOLAR SI",
            Units::MassSI => "MASS SI",
            Units::SIWithC => "SI WITH C",
            Units::MolarBaseSI => "MOLAR BASE SI",
            Units::MassBaseSI => "MASS BASE SI",
            Units::English => "ENGLISH",
            Units::MolarEnglish => "MOLAR ENGLISH",
            Units::MKS => "MKS",
            Units::CGS => "CGS",
            Units::Mixed => "MIXED",
            Units::MEUnits => "MEUNITS",
            // Units::User => "USER",
            Units::User => unimplemented!(),
            // Units::Custom(s) => s.as_str(),
            Units::Custom(_) => unimplemented!(),
        };

        // Use iFlag=0 to check all strings possible
        RefpropFunctionLibrary::get_enum(GetEnumFlag::UnitsOnly, enum_str)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Basis {
    /// All inputs and outputs are given on a mole basis.
    Molar = 0,
    /// All inputs and outputs are given on a mass basis.
    Mass = 1,
    /// All inputs and outputs are given on a mass basis except composition.
    MassExceptComposition = 2,
}

#[derive(Debug, Clone, Copy)]
pub enum Phase {
    /// Unknown phase; saturation routines will determine the phase.
    Unknown = 0,
    /// State point is in the liquid phase.
    Liquid = 1,
    /// State point is in the vapor phase.
    Vapor = 2,
    /// State point is in the two-phase region.
    TwoPhase = 3,
}

/// Represents the kr/kq flags for the `ab_fls_h` method.
#[derive(Debug, Clone, Copy)]
pub enum KrKqFlag {
    /// Default flag.
    Default = 0,
    /// Quality on a molar basis (moles vapor/total moles).
    QualityMolar = 1,
    /// Quality on a mass basis (mass vapor/total mass).
    QualityMass = 2,
    /// Return lower density root.
    LowerDensity = 3,
    /// Return higher density root.
    HigherDensity = 4,
}

/// A safe wrapper around the REFPROP library.
pub struct RefpropFunctionLibrary;
