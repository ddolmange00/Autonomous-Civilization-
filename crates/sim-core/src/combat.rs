use crate::monsters::MonsterSpecies;

#[derive(Clone, Copy, Debug, Default)]
pub struct Force {
    pub fighters: f32,
    pub ranged_power: f32,
    pub impact_power: f32,
    pub piercing_power: f32,
    pub armor: f32,
    pub mobility: f32,
    pub morale: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CombatOutcome {
    pub human_losses: f32,
    pub monster_losses: f32,
    pub settlement_damage: f32,
    pub monsters_repulsed: bool,
}

pub fn resolve(force: Force, monster: &MonsterSpecies, monsters: f32) -> CombatOutcome {
    let monster_power = monsters.max(0.0)
        * monster.body_mass_kg.sqrt()
        * (0.4 + monster.aggression)
        * (0.6 + monster.armor)
        * (0.7 + monster.speed_m_s / 20.0);
    let anti_armor = force.impact_power * 0.55 + force.piercing_power * 0.75;
    let human_power = force.fighters.sqrt()
        * (force.ranged_power + anti_armor + force.mobility * 0.25)
        * (0.5 + force.morale)
        * (0.7 + force.armor * 0.3);
    let ratio = human_power / monster_power.max(0.01);
    let repulsed = ratio >= 1.0;
    CombatOutcome {
        human_losses: (monster_power / (human_power + monster_power) * force.fighters * 0.45).min(force.fighters),
        monster_losses: (human_power / (human_power + monster_power) * monsters * 0.65).min(monsters),
        settlement_damage: if repulsed { (1.0 / (1.0 + ratio)) * 0.25 } else { (1.0 - ratio.min(1.0)) * 0.8 + 0.15 },
        monsters_repulsed: repulsed,
    }
}
