use sim_core::{
    environment::Barrier,
    traversal::{can_cross,Capabilities},
    monster_ai::{decide,MonsterContext,MonsterState},
};

#[test]
fn no_raft_by_calendar_magic() {
    for _year in [1,100,1000,10000] {
        assert!(!can_cross(Barrier::DeepWater,Capabilities::default(),0.4));
    }
}

#[test]
fn food_rich_wildlife_can_divert_monster_from_village() {
    let state=decide(MonsterContext{
        hunger:0.7,wildlife_food:0.9,settlement_food:0.8,distance_to_settlement:2.,
        perceived_defense:0.1,aggression:0.8,intelligence:0.5,health:1.
    });
    assert_eq!(state,MonsterState::HuntWildlife);
}
