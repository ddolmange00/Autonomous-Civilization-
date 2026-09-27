use sim_core::{
    environment::Barrier,
    traversal::{can_cross,Capabilities},
    monster_ai::{decide,MonsterContext,MonsterState},
};

#[test]
fn no_raft_by_calendar_magic() {
    for _year in [1,100,1000,10000] {
        assert!(!can_cross(Barrier::DeepWater,Capabilities::default(),.4));
    }
}

#[test]
fn food_rich_wildlife_can_divert_monster_from_village() {
    let state=decide(MonsterContext{
        hunger:.7,wildlife_food:.9,settlement_food:.8,distance_to_settlement:2.,
        perceived_defense:.1,aggression:.8,intelligence:.5,health:1.
    });
    assert_eq!(state,MonsterState::HuntWildlife);
}
