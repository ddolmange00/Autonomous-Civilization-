use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonsterState { Roam, HuntWildlife, ApproachSettlement, Raid, Fight, Retreat, Starve }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct MonsterContext {
    pub hunger: f32,
    pub wildlife_food: f32,
    pub settlement_food: f32,
    pub distance_to_settlement: f32,
    pub perceived_defense: f32,
    pub aggression: f32,
    pub intelligence: f32,
    pub health: f32,
}

pub fn decide(c: MonsterContext) -> MonsterState {
    if c.health < 0.2 && c.perceived_defense > 0.4 { return MonsterState::Retreat; }
    if c.hunger > 0.85 && c.wildlife_food < 0.1 && c.settlement_food < 0.1 { return MonsterState::Starve; }
    if c.wildlife_food > 0.35 && c.hunger > 0.35 { return MonsterState::HuntWildlife; }
    let raid_drive = c.hunger * 0.45 + c.aggression * 0.35 + c.intelligence * 0.20;
    let risk = c.perceived_defense * (0.7 + c.intelligence * 0.3);
    if c.distance_to_settlement < 1.0 && raid_drive > risk { return MonsterState::Raid; }
    if c.distance_to_settlement < 12.0 && raid_drive > risk * 0.75 { return MonsterState::ApproachSettlement; }
    MonsterState::Roam
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn starving_aggressive_monster_can_choose_settlement() {
        let s=decide(MonsterContext{hunger:.95,wildlife_food:.02,settlement_food:.9,distance_to_settlement:5.,perceived_defense:.1,aggression:.9,intelligence:.4,health:1.});
        assert_eq!(s,MonsterState::ApproachSettlement);
    }
    #[test] fn wounded_monster_can_retreat() {
        let s=decide(MonsterContext{hunger:.8,wildlife_food:.1,settlement_food:.8,distance_to_settlement:.5,perceived_defense:.9,aggression:.8,intelligence:.7,health:.1});
        assert_eq!(s,MonsterState::Retreat);
    }
}
