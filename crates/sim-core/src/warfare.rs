use crate::{
    combat::{resolve, CombatOutcome, Force},
    design::{evaluate, DesignGenome},
    materials::MaterialProperties,
    monsters::MonsterSpecies,
};
use serde::{Deserialize, Serialize};

/// A self-organized fighting group: residents who chose Attack near the same
/// threat. No appointed leader, no scripted war stages; morale is memory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Warband {
    pub id: u64,
    pub settlement_id: Option<u64>,
    pub members: Vec<u64>,
    pub morale: f32,
    pub target_monster: Option<u64>,
    pub battles: u32,
}

/// Group attackers by nearest threat. Returns (member_ids, monster_id) clusters
/// with at least two members; lone fighters stay individual scratches.
pub fn form_warbands(
    attackers: &[(u64, f32, f32, Option<u64>)],
    monsters: &[(u64, f32, f32)],
    radius: f32,
) -> Vec<(Vec<u64>, u64)> {
    let mut groups: Vec<(Vec<u64>, u64)> = Vec::new();
    for (rid, x, y, _) in attackers {
        let mut best: Option<(usize, f32)> = None;
        for m in monsters {
            let d = ((x - m.1).powi(2) + (y - m.2).powi(2)).sqrt();
            if d <= radius && best.map(|(_, bd)| d < bd).unwrap_or(true) {
                if let Some(gi) = groups.iter().position(|(_, mid)| *mid == m.0) {
                    best = Some((gi, d));
                } else {
                    groups.push((Vec::new(), m.0));
                    best = Some((groups.len() - 1, d));
                }
            }
        }
        if let Some((gi, _)) = best {
            groups[gi].0.push(*rid);
        }
    }
    groups.into_iter().filter(|(m, _)| m.len() >= 2).collect()
}

/// Melee power from the best known purpose-built weapon in the repertoire.
/// Shelter sticks count for almost nothing: tools must be invented.
pub fn melee_from_repertoire(designs: &[DesignGenome], truth: &MaterialProperties) -> f32 {
    use crate::design::Function;
    designs
        .iter()
        .filter(|d| matches!(d.function, Function::Cut | Function::Pierce | Function::Impact))
        .map(|d| {
            let p = evaluate(d, *truth);
            p.cutting + p.piercing * 0.7 + p.impact * 0.5
        })
        .fold(0.0_f32, f32::max)
}

pub fn assemble_force(band: &Warband, melee: f32, armor: f32) -> Force {
    Force {
        fighters: band.members.len() as f32,
        ranged_power: 0.0,
        impact_power: melee * 0.5,
        piercing_power: melee * 0.6,
        armor,
        mobility: 0.5,
        morale: band.morale.clamp(0.0, 1.0),
    }
}

pub fn resolve_battle(force: Force, species: &MonsterSpecies, monsters: f32) -> CombatOutcome {
    resolve(force, species, monsters)
}

/// Morale remembers: repulsion emboldens, losses and routs break.
/// Bands below spirit disband into fleeing individuals.
pub fn update_morale(band: &mut Warband, outcome: &CombatOutcome) {
    if outcome.monsters_repulsed {
        band.morale = (band.morale + 0.15).min(1.0);
    } else {
        let loss_ratio = (outcome.human_losses / band.members.len().max(1) as f32).clamp(0.0, 1.0);
        band.morale = (band.morale - 0.10 - loss_ratio * 0.35).max(0.0);
    }
    band.battles += 1;
}

pub fn disbanded(band: &Warband) -> bool {
    band.members.len() < 2 || band.morale < 0.15
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::Function;
    #[test]
    fn lone_attackers_form_no_warband() {
        let a = vec![(1u64, 0.0, 0.0, None)];
        let m = vec![(9u64, 5.0, 0.0)];
        assert!(form_warbands(&a, &m, 60.0).is_empty());
    }
    #[test]
    fn neighbors_against_one_threat_band_together() {
        let a = vec![(1u64, 0.0, 0.0, None), (2u64, 4.0, 0.0, None), (3u64, 200.0, 0.0, None)];
        let m = vec![(9u64, 5.0, 0.0)];
        let g = form_warbands(&a, &m, 60.0);
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].0.len(), 2);
        assert_eq!(g[0].1, 9);
    }
    #[test]
    fn sticks_are_not_weapons() {
        let stick = DesignGenome {
            id: 1,
            parent: None,
            generation: 0,
            function: Function::Shelter,
            length_m: 2.0,
            width_m: 2.0,
            thickness_m: 0.1,
            curvature: 0.1,
            edge_fraction: 0.0,
            binding_quality: 0.5,
            material_fraction: vec![],
        };
        let truth = MaterialProperties {
            density: 0.6,
            tensile_strength: 0.55,
            compressive_strength: 0.45,
            fracture_toughness: 0.5,
            hardness: 0.3,
            elastic_modulus: 0.5,
            flexibility: 0.5,
            friction: 0.6,
            thermal_conductivity: 0.3,
            ignition_temperature: 0.6,
            corrosion_resistance: 0.5,
            water_absorption: 0.5,
            permeability: 0.3,
            buoyancy_factor: 0.9,
            workability: 0.8,
        };
        assert_eq!(melee_from_repertoire(&[stick.clone()], &truth), 0.0);
        let blade = DesignGenome { function: Function::Cut, edge_fraction: 0.8, ..stick.clone() };
        assert!(melee_from_repertoire(&[blade], &truth) > 0.1);
    }
    #[test]
    fn morale_remembers_outcomes() {
        let mut b = Warband { id: 1, settlement_id: None, members: vec![1, 2, 3], morale: 0.5, target_monster: None, battles: 0 };
        update_morale(&mut b, &CombatOutcome { human_losses: 0.0, monster_losses: 1.0, settlement_damage: 0.0, monsters_repulsed: true });
        assert!(b.morale > 0.5 && b.battles == 1);
        update_morale(&mut b, &CombatOutcome { human_losses: 2.0, monster_losses: 0.0, settlement_damage: 0.5, monsters_repulsed: false });
        assert!(b.morale < 0.65);
        b.morale = 0.1;
        assert!(disbanded(&b));
    }
}
