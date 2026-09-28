use sim_core::{
    generation::{conception_propensity,ReproductionContext},
    life_history::{LifeStage,Sex},
    relationships::Relation,
};

#[test]
fn reproduction_is_not_population_threshold_based() {
    let good=conception_propensity(ReproductionContext{
        stage:LifeStage::Adult,sex:Sex::Female,health:1.0,hunger:0.1,safety_need:0.1,care_trait:0.8,
        household_pressure:0.1,partner_relation:Relation{trust:0.8,affection:0.8,..Default::default()}
    });
    let stressed=conception_propensity(ReproductionContext{
        stage:LifeStage::Adult,sex:Sex::Female,health:0.5,hunger:0.9,safety_need:0.9,care_trait:0.8,
        household_pressure:0.95,partner_relation:Relation{trust:0.8,affection:0.8,..Default::default()}
    });
    assert!(good>stressed);
}
#[test]
fn children_do_not_conceive() {
    let x=conception_propensity(ReproductionContext{stage:LifeStage::Child,sex:Sex::Female,health:1.0,hunger:0.0,safety_need:0.0,care_trait:1.0,household_pressure:0.0,partner_relation:Relation{trust:1.0,affection:1.0,..Default::default()}});
    assert_eq!(x,0.0);
}
