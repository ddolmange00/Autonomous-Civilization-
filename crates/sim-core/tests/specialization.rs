use sim_core::{agency::ActionPrimitive,specialization::PracticeProfile};

#[test]
fn repeated_actions_create_specialization() {
    let mut p=PracticeProfile::default();
    for i in 0..120 {p.practice(ActionPrimitive::Dig,0.8,1.0,i as f64/365.0);}
    for i in 0..8 {p.practice(ActionPrimitive::Attack,0.3,1.0,i as f64/365.0);}
    assert!(p.skill(ActionPrimitive::Dig)>p.skill(ActionPrimitive::Attack));
    assert_eq!(p.dominant(1)[0].0,ActionPrimitive::Dig);
}
