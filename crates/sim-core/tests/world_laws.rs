use sim_core::{
    hydrology::{step_bucket,WaterBalance,WaterFlux},
    soil::Soil,
    structures::euler_buckling_load_n,
};

#[test]
fn water_balance_never_creates_water_from_nothing() {
    let b=step_bucket(WaterBalance{surface_water_mm:10.,soil_water_mm:20.,groundwater_mm:5.},
        WaterFlux{rainfall_mm:0.,infiltration_capacity_mm:100.,evapotranspiration_mm:4.,drainage_fraction:0.2});
    assert!(b.surface_water_mm+b.soil_water_mm+b.groundwater_mm<=35.0+1e-9);
}

#[test]
fn saturated_slope_is_not_safer_than_dry_equivalent() {
    let base=Soil{sand:0.4,silt:0.4,clay:0.2,organic_matter:0.03,depth_m:2.,porosity:0.45,field_capacity:0.3,wilting_point:0.12,
        saturation:0.,hydraulic_conductivity_m_day:0.1,cohesion_kpa:8.,friction_angle_deg:30.};
    let mut wet=base; wet.saturation=1.;
    assert!(wet.slope_safety_factor(0.6,18.)<=base.slope_safety_factor(0.6,18.));
}

#[test]
fn doubling_column_length_quarters_euler_capacity() {
    let a=euler_buckling_load_n(8e9,2e-5,1.);
    let b=euler_buckling_load_n(8e9,2e-5,2.);
    assert!((a/b-4.).abs()<1e-9);
}
