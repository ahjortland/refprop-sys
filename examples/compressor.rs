use refprop_sys::{Basis, KrKqFlag, Phase, RefpropError, RefpropFunctionLibrary};

fn main() -> Result<(), RefpropError> {
    // Set path and initilaize fluid.
    let _ = RefpropFunctionLibrary::set_path(None);
    let z = RefpropFunctionLibrary::set_mixture("R454B")?;

    // Define flags: Molar basis, Two-phase, Default kr/kq
    let imass = Basis::Molar;
    let kph = Phase::Unknown;
    let krkq = KrKqFlag::QualityMolar;

    // Perform flash calculation
    let suction = RefpropFunctionLibrary::tq_flash(290.0, 1.0, &z, imass, kph, krkq)?;
    let p_dis = suction.P * 2.9;
    let s_dis_s = suction.s;
    let discharge_s = RefpropFunctionLibrary::ab_flash(
        "PS",
        p_dis,
        s_dis_s,
        &z,
        imass,
        Phase::Vapor,
        KrKqFlag::Default,
    )?;
    let h_dis = suction.h + (discharge_s.h - suction.h) / 0.7;
    let discharge = RefpropFunctionLibrary::ab_flash("PH", p_dis, h_dis, &z, imass, kph, krkq)?;

    let w_cmp = discharge.h - suction.h;

    println!("{}", w_cmp);

    Ok(())
}
