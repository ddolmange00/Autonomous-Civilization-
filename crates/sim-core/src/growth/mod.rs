//! Growth trees (`docs/PC_WORLDBOX_VISION.md`, "Growth trees"): authored,
//! regional capability trees. Content lives in `content/growth/*.ron` and is
//! embedded at build time; this module parses and checks it and answers what
//! a tribe can discover, learn from contact, and build.
pub mod catalog;
pub mod eval;
pub mod region;
pub mod runtime;
pub mod schema;

pub use catalog::{BitSet, Catalog, CatalogError, NodeId, PlaceSet, ResourceId, UnlockId};
pub use eval::{DiscoveryParams, MenuEntry, Missing, Tribe};

macro_rules! content {
    ($name:literal) => { ($name, include_str!(concat!("../../content/growth/", $name))) };
}

pub const RESOURCES: (&str, &str) = content!("resources.ron");
pub const DOMAINS: [(&str, &str); 10] = [
    content!("food.ron"),
    content!("tools_weapons.ron"),
    content!("armour.ron"),
    content!("architecture.ron"),
    content!("religion.ron"),
    content!("professions.ron"),
    content!("materials.ron"),
    content!("transport.ron"),
    content!("medicine.ron"),
    content!("governance.ron"),
];

/// The catalog shipped with the game.
pub fn builtin() -> Result<Catalog, Vec<CatalogError>> {
    Catalog::parse(RESOURCES, &DOMAINS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::{CultureLean, Place, Spur};

    const RES: &str = r#"(resources: [
        (id: "wood", name: "Wood", category: Wood, sources: [(place: Biome(TemperateForest))]),
        (id: "copper_ore", name: "Copper ore", category: Ore, sources: [(place: Hills)]),
        (id: "tin_ore", name: "Tin ore", category: Ore, sources: [(place: Mountain)]),
        (id: "bronze", name: "Bronze", category: Metal),
    ])"#;
    const NODES: &str = r#"(domain: Materials, nodes: [
        (id: "fire", name: "Fire", tier: 0, era: "stone-using", unlocks: [
            (id: "hut", name: "Hut", kind: Building, cost: [("wood", 10)], footprint: (2, 2),
             variants: [
                (when: [Biome(Desert)], name: "Mud hut", look: "mud"),
                (lean: Construction, name: "Longhouse", look: "longhouse", cost: [("wood", 20)]),
             ]),
        ]),
        (id: "smelting", name: "Smelting", tier: 1, requires: ["fire"], needs: ["copper_ore"]),
        (id: "bronze_casting", name: "Bronze casting", tier: 2, requires: ["smelting"],
         needs: ["copper_ore", "tin_ore"], era: "bronze-casting", spurs: [War], unlocks: [
            (id: "alloying", name: "Alloying", kind: Process,
             consumes: [("copper_ore", 3), ("tin_ore", 1)], produces: [("bronze", 4)]),
            (id: "foundry", name: "Foundry", kind: Building, cost: [("wood", 30), ("bronze", 5)],
             conditions: [Population(40), Building("hut")]),
        ]),
        (id: "canoes", name: "Canoes", tier: 1, requires: ["fire"], local: [Coast, River, Lake]),
    ])"#;

    fn cat() -> Catalog { Catalog::parse(("res", RES), &[("nodes", NODES)]).expect("test catalog compiles") }

    fn tribe(c: &Catalog, places: &[Place], local: &[&str], known: &[&str]) -> Tribe {
        let mut t = Tribe { places: PlaceSet::of(places), population: 30, ..Tribe::default() };
        for r in local { t.local.insert(c.resource(r).unwrap()); }
        for n in known { t.known.insert(c.node(n).unwrap()); }
        t
    }

    #[test]
    fn missing_tin_blocks_bronze_until_it_is_imported() {
        let c = cat();
        let bronze = c.node("bronze_casting").unwrap();
        let mut t = tribe(&c, &[Place::Hills], &["copper_ore"], &["fire", "smelting"]);
        let access = c.tribe_access(&t);
        assert_eq!(c.discovery_gaps(&t, bronze, &access), vec![Missing::Access(c.resource("tin_ore").unwrap())]);
        assert_eq!(c.discovery_rate(&t, bronze, &access, DiscoveryParams::default()), 0.0);

        t.imported.insert(c.resource("tin_ore").unwrap());
        let access = c.tribe_access(&t);
        assert!(c.discovery_gaps(&t, bronze, &access).is_empty());
        assert!(c.discovery_rate(&t, bronze, &access, DiscoveryParams::default()) > 0.0);
        // Knowing the process turns the imported ore into bronze access.
        assert!(!access.contains(c.resource("bronze").unwrap()));
        t.known.insert(bronze);
        assert!(c.tribe_access(&t).contains(c.resource("bronze").unwrap()));
    }

    #[test]
    fn inland_tribe_cannot_invent_canoes_but_can_learn_them() {
        let c = cat();
        let canoes = c.node("canoes").unwrap();
        let inland = tribe(&c, &[Place::Biome(schema::BiomeTag::Desert)], &[], &["fire"]);
        let access = c.tribe_access(&inland);
        assert!(matches!(c.discovery_gaps(&inland, canoes, &access)[..], [Missing::Place(_)]));
        let coastal = tribe(&c, &[Place::Coast], &[], &["fire", "canoes"]);
        assert_eq!(c.learnable_from(&inland.known, &coastal.known), vec![canoes]);
        // Prerequisites still apply to learning: without fire, only fire itself.
        assert_eq!(c.learnable_from(&BitSet::default(), &coastal.known), vec![c.node("fire").unwrap()]);
    }

    #[test]
    fn trials_speed_up_discovery_and_acceleration_scales_it() {
        let c = cat();
        let bronze = c.node("bronze_casting").unwrap();
        let mut t = tribe(&c, &[Place::Hills, Place::Mountain], &["copper_ore", "tin_ore"], &["fire", "smelting"]);
        let access = c.tribe_access(&t);
        let p = DiscoveryParams::default();
        let calm = c.discovery_rate(&t, bronze, &access, p);
        t.survived(Spur::War);
        let pressed = c.discovery_rate(&t, bronze, &access, p);
        assert!(pressed > calm * 2.0);
        let rushed = c.discovery_rate(&t, bronze, &access, DiscoveryParams { acceleration: 10.0, ..p });
        assert!((rushed / pressed - 10.0).abs() < 1e-3);
    }

    #[test]
    fn renditions_follow_land_then_culture() {
        let c = cat();
        let hut = c.unlock("hut").unwrap();
        let mut t = tribe(&c, &[Place::Biome(schema::BiomeTag::Desert)], &[], &["fire"]);
        assert_eq!(c.rendition(hut, &t).0, "Mud hut");
        t.places = PlaceSet::of(&[Place::Biome(schema::BiomeTag::Grassland)]);
        assert_eq!(c.rendition(hut, &t).0, "Hut");
        t.lean = Some(CultureLean::Construction);
        let (name, look, cost) = c.rendition(hut, &t);
        assert_eq!((name, look, cost[0].1), ("Longhouse", "longhouse", 20.0));
    }

    #[test]
    fn build_menu_explains_every_gap_and_clears_when_met() {
        let c = cat();
        let (wood, bronze_r) = (c.resource("wood").unwrap(), c.resource("bronze").unwrap());
        let foundry = c.unlock("foundry").unwrap();
        let hut = c.unlock("hut").unwrap();
        let mut t = tribe(&c, &[Place::Hills, Place::Mountain], &["copper_ore", "tin_ore"], &["fire", "smelting", "bronze_casting"]);
        let entry = |t: &Tribe| c.build_menu(t).into_iter().find(|e| e.unlock == foundry).unwrap().missing;
        assert_eq!(entry(&t), vec![
            Missing::Access(wood),
            Missing::Stock { resource: bronze_r, have: 0.0, need: 5.0 },
            Missing::Population { have: 30, need: 40 },
            Missing::Has(hut),
        ]);
        t.imported.insert(wood);
        t.stock = vec![0.0; c.resources.len()];
        t.stock[wood as usize] = 30.0;
        t.stock[bronze_r as usize] = 5.0;
        t.population = 40;
        t.have = vec![0; c.unlocks.len()];
        t.have[hut as usize] = 1;
        assert!(entry(&t).is_empty());
    }

    #[test]
    fn era_is_named_by_the_deepest_labelled_node() {
        let c = cat();
        let t = tribe(&c, &[], &[], &["fire", "smelting"]);
        assert_eq!(c.era(&t.known), Some("stone-using"));
        let t = tribe(&c, &[], &[], &["fire", "smelting", "bronze_casting"]);
        assert_eq!(c.era(&t.known), Some("bronze-casting"));
    }

    #[test]
    fn validation_rejects_bad_references_tiers_and_dead_nodes() {
        let bad = r#"(domain: Materials, nodes: [
            (id: "a", name: "A", tier: 0, unlocks: [(id: "x", name: "X", kind: Building, cost: [("nothing", 1)])]),
            (id: "a", name: "A again", tier: 1),
            (id: "b", name: "B", tier: 0, requires: ["a"]),
            (id: "c", name: "C", tier: 2, requires: ["ghost"]),
            (id: "d", name: "D", tier: 1, requires: ["a"], unlocks: [(id: "y", name: "Y", kind: Tool, conditions: [Building("z")])]),
            (id: "e", name: "E", tier: 1, unlocks: [(id: "z", name: "Z", kind: Job)]),
        ])"#;
        let errs = Catalog::parse(("res", RES), &[("bad", bad)]).unwrap_err();
        let has = |f: &dyn Fn(&CatalogError) -> bool| errs.iter().any(f);
        assert!(has(&|e| matches!(e, CatalogError::Unknown { kind: "resource", id, .. } if id == "nothing")));
        assert!(has(&|e| matches!(e, CatalogError::Duplicate { kind: "node", id } if id == "a")));
        assert!(has(&|e| matches!(e, CatalogError::TierOrder { node, .. } if node == "b")));
        assert!(has(&|e| matches!(e, CatalogError::Unknown { kind: "node", id, .. } if id == "ghost")));
        assert!(has(&|e| matches!(e, CatalogError::WrongKind { id, .. } if id == "z")));

        // Compiles structurally, but the node needs a resource only it can produce.
        let dead = r#"(domain: Materials, nodes: [
            (id: "fire", name: "Fire", tier: 0),
            (id: "loop", name: "Loop", tier: 1, requires: ["fire"], needs: ["bronze"], unlocks: [
                (id: "p", name: "P", kind: Process, produces: [("bronze", 1)]),
            ]),
        ])"#;
        let errs = Catalog::parse(("res", RES), &[("dead", dead)]).unwrap_err();
        assert!(errs.contains(&CatalogError::Unreachable { node: "loop".into() }));
        assert!(errs.contains(&CatalogError::Unobtainable { resource: "bronze".into() }));

        let typo = r#"(domain: Materials, nodes: [(id: "f", name: "F", tier: 0, tierr: 1)])"#;
        assert!(matches!(Catalog::parse(("res", RES), &[("typo", typo)]).unwrap_err()[..], [CatalogError::Parse { .. }]));
    }
}
