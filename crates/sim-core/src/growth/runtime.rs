//! Growth trees inside the running world. Each settlement is a tribe; what it
//! knows is exactly what its living members carry as `growth::<node>`
//! knowledge keys, so knowledge spreads through the existing teaching paths
//! and dies with its last carrier. Each tick derives the rest from the
//! sandbox: region, population, cultural lean, recent hardships, imports
//! from trade partners. Then it rolls independent discovery and learning
//! from trade contacts.
use super::catalog::{BitSet, Catalog, NodeId, PlaceSet};
use super::eval::{DiscoveryParams, Tribe};
use super::region::{self, local_resources, places_around, region_key};
use super::schema::{BiomeTag, CultureLean, Place, Spur};
use crate::{
    causal_log::CausalNode, events::WorldEventKind, life_history::LifeStage, sandbox::Sandbox,
    terrain::TileMap, world::Position,
};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, OnceLock};

pub const KEY_PREFIX: &str = "growth::";
/// Growth is evaluated at this cadence, not every simulation step.
pub const TICK_YEARS: f64 = 0.25;
/// A hardship keeps spurring discovery for this long after it was felt.
const TRIAL_MEMORY_YEARS: f64 = 10.0;
/// A trade partner's goods keep arriving for this long after the last exchange.
const IMPORT_MEMORY_YEARS: f64 = 5.0;
const REGION_RADIUS_TILES: usize = 24;
/// Chance that one exchange carries one learnable idea across.
const LEARN_PER_TRADE: f32 = 0.15;
const THREAT_RADIUS: f32 = 150.0;
const CROWDED_POPULATION: usize = 80;
/// A carried key below this confidence is a rumour, not know-how.
const KNOW_CONFIDENCE: f32 = 0.05;

pub fn shared_catalog() -> Arc<Catalog> {
    static CATALOG: OnceLock<Arc<Catalog>> = OnceLock::new();
    CATALOG
        .get_or_init(|| Arc::new(super::builtin().expect("built-in growth content is validated by tests/growth_catalog.rs")))
        .clone()
}

pub fn key(c: &Catalog, n: NodeId) -> String { format!("{KEY_PREFIX}{}", c.nodes[n as usize].id) }

#[derive(Clone, Debug)]
pub struct TribeGrowth {
    pub tribe: Tribe,
    /// Player acceleration for this tribe (1 = none).
    pub acceleration: f32,
    pub discoveries: Vec<(f64, NodeId)>,
    region_tile: Option<(usize, usize)>,
    spur_seen: [f64; Spur::ALL.len()],
    /// (year of last exchange, partner settlement id)
    partners: Vec<(f64, u64)>,
}

impl TribeGrowth {
    fn new() -> Self {
        Self {
            tribe: Tribe::default(),
            acceleration: 1.0,
            discoveries: Vec::new(),
            region_tile: None,
            spur_seen: [f64::NEG_INFINITY; Spur::ALL.len()],
            partners: Vec::new(),
        }
    }
    fn feel(&mut self, s: Spur, year: f64) { self.spur_seen[s as usize] = year; }
}

#[derive(Clone, Debug)]
pub struct GrowthWorld {
    pub catalog: Arc<Catalog>,
    /// Keyed by settlement id.
    pub tribes: BTreeMap<u64, TribeGrowth>,
    pub params: DiscoveryParams,
    /// Player acceleration by settlement id. Kept after a settlement fades so
    /// the settlements that split from it inherit the setting.
    acceleration: BTreeMap<u64, f32>,
    /// Ideas acquired world-wide: (independent discoveries, learned from contact).
    pub acquired: (u64, u64),
    last_tick: f64,
    ticks: u64,
}

impl Default for GrowthWorld {
    fn default() -> Self {
        Self {
            catalog: shared_catalog(), tribes: BTreeMap::new(), params: DiscoveryParams::default(),
            acceleration: BTreeMap::new(), acquired: (0, 0), last_tick: 0.0, ticks: 0,
        }
    }
}

impl GrowthWorld {
    /// Player lever: speed up (or slow down) one tribe's discoveries.
    /// Settlements that later split from this one inherit the factor.
    pub fn set_acceleration(&mut self, settlement_id: u64, factor: f32) {
        let f = factor.max(0.0);
        self.acceleration.insert(settlement_id, f);
        if let Some(t) = self.tribes.get_mut(&settlement_id) { t.acceleration = f; }
    }
    pub fn era(&self, settlement_id: u64) -> Option<&str> {
        self.tribes.get(&settlement_id).and_then(|t| self.catalog.era(&t.tribe.known))
    }
}

/// Advance growth if a tick is due. Call once per sandbox step.
pub fn step(sb: &mut Sandbox) {
    if sb.year < sb.growth.last_tick + TICK_YEARS { return; }
    let mut g = std::mem::take(&mut sb.growth);
    let dt = (sb.year - g.last_tick).min(5.0) as f32;
    g.tick(sb, dt);
    g.last_tick = sb.year;
    g.ticks += 1;
    sb.growth = g;
}

fn abstract_places() -> PlaceSet {
    PlaceSet::of(&[Place::Biome(BiomeTag::Grassland), Place::Biome(BiomeTag::TemperateForest), Place::River, Place::Hills])
}

fn lean_of(p: crate::culture::EmergentProfile) -> Option<CultureLean> {
    [
        (CultureLean::Confrontation, p.confrontation),
        (CultureLean::Avoidance, p.avoidance),
        (CultureLean::Experimentation, p.experimentation),
        (CultureLean::Cooperation, p.cooperation),
        (CultureLean::Construction, p.construction),
    ]
    .into_iter()
    .filter(|(_, v)| *v > 0.05)
    .max_by(|a, b| a.1.total_cmp(&b.1))
    .map(|(l, _)| l)
}

impl GrowthWorld {
    fn tick(&mut self, sb: &mut Sandbox, dt: f32) {
        let c = self.catalog.clone();
        let year = sb.year;
        let by_id: HashMap<u64, usize> = sb.residents.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
        let starting: Vec<NodeId> = (0..c.nodes.len() as NodeId).filter(|&n| c.nodes[n as usize].starting).collect();

        self.tribes.retain(|id, _| sb.settlements.iter().any(|s| s.id == *id && !s.members.is_empty()));

        for s in &sb.settlements {
            let living: Vec<usize> = s.members.iter().filter_map(|id| by_id.get(id).copied()).filter(|&i| sb.residents[i].health > 0.0).collect();
            if living.is_empty() { continue; }
            let fresh = !self.tribes.contains_key(&s.id);
            let inherited = s.parent_id.and_then(|p| self.acceleration.get(&p).copied());
            if let (true, Some(f)) = (fresh, inherited) { self.acceleration.entry(s.id).or_insert(f); }
            let tg = self.tribes.entry(s.id).or_insert_with(TribeGrowth::new);
            tg.acceleration = self.acceleration.get(&s.id).copied().unwrap_or(1.0);
            if fresh {
                // A founding band carries the basics of survival.
                for &i in &living {
                    for &n in &starting { sb.residents[i].knowledge.learn(key(&c, n), 1.0, 0.9); }
                }
            }
            refresh_region(tg, &c, sb.terrain.as_deref(), s.center, sb.seed);

            let t = &mut tg.tribe;
            t.population = living.len() as u32;
            t.lean = lean_of(s.profile());
            t.known = BitSet::with_len(c.nodes.len());
            for &i in &living {
                for (k, item) in &sb.residents[i].knowledge.items {
                    if item.confidence < KNOW_CONFIDENCE { continue; }
                    if let Some(n) = k.strip_prefix(KEY_PREFIX).and_then(|id| c.node(id)) { t.known.insert(n); }
                }
            }

            // Hardships felt now.
            let hunger = living.iter().map(|&i| sb.residents[i].mind.needs.hunger).sum::<f32>() / living.len() as f32;
            if hunger > 0.75 { tg.feel(Spur::Famine, year); }
            if living.len() > CROWDED_POPULATION { tg.feel(Spur::Crowding, year); }
            if s.members.len() > living.len() { tg.feel(Spur::Death, year); }
            let near = |p: Position, r: f32| (p.x - s.center.x).powi(2) + (p.y - s.center.y).powi(2) <= r * r;
            if sb.monsters.iter().any(|m| m.health > 0.0 && near(m.position, THREAT_RADIUS)) { tg.feel(Spur::Monsters, year); }
            if sb.warbands.iter().any(|w| w.settlement_id == Some(s.id) && w.battles > 0) { tg.feel(Spur::War, year); }
            for e in sb.events.iter().filter(|e| e.active(year) && near(e.position, THREAT_RADIUS + e.radius)) {
                match e.kind {
                    WorldEventKind::Flood => tg.feel(Spur::Flood, year),
                    WorldEventKind::Drought => tg.feel(Spur::Drought, year),
                    WorldEventKind::Fire => tg.feel(Spur::Heat, year),
                    WorldEventKind::Storm => tg.feel(Spur::Cold, year),
                    WorldEventKind::CreatureSpawn => tg.feel(Spur::Monsters, year),
                    _ => {}
                }
            }
            if let Some(map) = sb.terrain.as_deref() {
                if let Some((x, y)) = map.tile_at(s.center) {
                    let temp = map.temperature_c[map.index(x, y)];
                    if temp < 0 { tg.feel(Spur::Cold, year); }
                    if temp > 28 { tg.feel(Spur::Heat, year); }
                }
            }
        }

        // Trade since the last tick: partners, imports and ideas.
        let mut exchanges: Vec<(u64, u64)> = sb.trades.iter()
            .filter(|t| t.year > self.last_tick && t.year <= year)
            .map(|t| (t.from_settlement, t.to_settlement))
            .collect();
        exchanges.sort_unstable();
        for &(a, b) in &exchanges {
            for (me, other) in [(a, b), (b, a)] {
                if let Some(tg) = self.tribes.get_mut(&me) {
                    tg.feel(Spur::Trade, year);
                    tg.partners.retain(|(_, p)| *p != other);
                    tg.partners.push((year, other));
                }
            }
        }
        let access: HashMap<u64, BitSet> = self.tribes.iter().map(|(id, tg)| (*id, c.tribe_access(&tg.tribe))).collect();
        let known: HashMap<u64, BitSet> = self.tribes.iter().map(|(id, tg)| (*id, tg.tribe.known.clone())).collect();
        for tg in self.tribes.values_mut() {
            tg.partners.retain(|(y, p)| year - *y <= IMPORT_MEMORY_YEARS && access.contains_key(p));
            tg.tribe.imported = BitSet::with_len(c.resources.len());
            for (_, p) in &tg.partners { tg.tribe.imported.union_with(&access[p]); }
            tg.tribe.trials = Spur::ALL.iter()
                .filter(|s| year - tg.spur_seen[**s as usize] <= TRIAL_MEMORY_YEARS)
                .fold(0, |m, s| m | s.bit());
        }

        let seed = sb.seed ^ self.ticks.wrapping_mul(0x2545_F491_4F6C_DD1D);
        let mut learned: Vec<(u64, NodeId, &'static str)> = Vec::new();
        for (i, &(a, b)) in exchanges.iter().enumerate() {
            for (learner, teacher) in [(a, b), (b, a)] {
                let (Some(lk), Some(tk)) = (known.get(&learner), known.get(&teacher)) else { continue };
                let options = c.learnable_from(lk, tk);
                if options.is_empty() { continue; }
                if region::unit(seed, learner ^ (i as u64) << 20, teacher) < LEARN_PER_TRADE {
                    let pick = (region::unit(seed, teacher, learner ^ i as u64) * options.len() as f32) as usize;
                    learned.push((learner, options[pick.min(options.len() - 1)], "learned from trade partners"));
                }
            }
        }
        for (id, tg) in self.tribes.iter() {
            let acc = &access[id];
            let p = DiscoveryParams { acceleration: self.params.acceleration * tg.acceleration, ..self.params };
            for (n, gaps) in c.frontier(&tg.tribe) {
                if !gaps.is_empty() { continue; }
                let rate = c.discovery_rate(&tg.tribe, n, acc, p);
                let chance = 1.0 - (-rate * dt).exp();
                if region::unit(seed, *id, n as u64) < chance { learned.push((*id, n, "discovered")); }
            }
        }

        // Hand each new idea to a carrier: the most curious living adult.
        for (sid, n, how) in learned {
            let Some(s) = sb.settlements.iter().find(|s| s.id == sid) else { continue };
            let carrier = s.members.iter().filter_map(|id| by_id.get(id).copied())
                .filter(|&i| {
                    let r = &sb.residents[i];
                    r.health > 0.0 && matches!(r.life.stage(year), LifeStage::Adult | LifeStage::Elder)
                })
                .max_by(|&a, &b| sb.residents[a].mind.traits.curiosity.total_cmp(&sb.residents[b].mind.traits.curiosity));
            let Some(i) = carrier else { continue };
            let Some(tg) = self.tribes.get_mut(&sid) else { continue };
            if !tg.tribe.known.insert(n) { continue; }
            sb.residents[i].knowledge.learn(key(&c, n), 1.0, 0.8);
            tg.discoveries.push((year, n));
            if how == "discovered" { self.acquired.0 += 1 } else { self.acquired.1 += 1 }
            let rid = sb.residents[i].id;
            sb.causal_log.push(year, CausalNode::Outcome { resident_id: Some(rid), label: format!("{how} {}", c.nodes[n as usize].name), value: 1.0 });
        }
    }
}

fn refresh_region(tg: &mut TribeGrowth, c: &Catalog, map: Option<&TileMap>, center: Position, seed: u64) {
    let Some(map) = map else {
        if tg.region_tile.is_none() {
            tg.region_tile = Some((0, 0));
            tg.tribe.places = abstract_places();
            tg.tribe.local = local_resources(c, tg.tribe.places, region_key(seed, 0, 0));
        }
        return;
    };
    let Some((x, y)) = map.tile_at(center) else { return };
    let moved = tg.region_tile.is_none_or(|(ox, oy)| ox.abs_diff(x).max(oy.abs_diff(y)) > REGION_RADIUS_TILES / 2);
    if moved {
        tg.region_tile = Some((x, y));
        tg.tribe.places = places_around(map, x, y, REGION_RADIUS_TILES);
        tg.tribe.local = local_resources(c, tg.tribe.places, region_key(map.seed, x, y));
    }
}
