//! Per-tribe questions over the catalog: what is next to discover, what can
//! be learned from a contact, what can be built now and why not.
use super::catalog::*;
use super::schema::{CultureLean, Effect, Spur};

/// Growth-relevant state of one tribe. Dense vectors are indexed by the
/// catalog's resource and unlock ids; shorter vectors read as zero.
#[derive(Clone, Debug, Default)]
pub struct Tribe {
    pub known: BitSet,
    pub places: PlaceSet,
    /// Natural resources of the tribe's own region.
    pub local: BitSet,
    /// Resources arriving through trade, tribute or plunder.
    pub imported: BitSet,
    pub stock: Vec<f32>,
    pub population: u32,
    /// Buildings standing, or residents holding a job, per unlock.
    pub have: Vec<u32>,
    /// Trials survived (`Spur` bits).
    pub trials: u32,
    pub lean: Option<CultureLean>,
}

impl Tribe {
    pub fn stock_of(&self, r: ResourceId) -> f32 { self.stock.get(r as usize).copied().unwrap_or(0.0) }
    pub fn count_of(&self, u: UnlockId) -> u32 { self.have.get(u as usize).copied().unwrap_or(0) }
    pub fn survived(&mut self, s: Spur) { self.trials |= s.bit(); }
}

/// Why something is not available yet.
#[derive(Clone, Debug, PartialEq)]
pub enum Missing {
    Prerequisite(NodeId),
    AnyOf(Vec<NodeId>),
    /// Local discovery needs one of these places.
    Place(PlaceSet),
    Access(ResourceId),
    Stock { resource: ResourceId, have: f32, need: f32 },
    Population { have: u32, need: u32 },
    Has(UnlockId),
    Node(NodeId),
    Trial(Spur),
}

#[derive(Clone, Debug)]
pub struct MenuEntry<'a> {
    pub unlock: UnlockId,
    pub name: &'a str,
    pub look: &'a str,
    pub cost: &'a [(ResourceId, f32)],
    pub missing: Vec<Missing>,
}

impl MenuEntry<'_> {
    pub fn ready(&self) -> bool { self.missing.is_empty() }
}

/// Discovery pacing. The constants are starting points; the sandbox wiring
/// calibrates them against long no-intervention runs.
#[derive(Clone, Copy, Debug)]
pub struct DiscoveryParams {
    /// Hazard per year for an effort-1, tier-0 node in a 30-person tribe.
    pub base_per_year: f32,
    /// Player acceleration for this tribe (1 = none).
    pub acceleration: f32,
}

impl Default for DiscoveryParams {
    fn default() -> Self { Self { base_per_year: 0.03, acceleration: 1.0 } }
}

impl Catalog {
    pub fn tribe_access(&self, t: &Tribe) -> BitSet {
        let mut raw = t.local.clone();
        raw.union_with(&t.imported);
        self.access(&t.known, t.places, &raw)
    }

    fn prerequisite_gaps(&self, n: NodeId, known: &BitSet, out: &mut Vec<Missing>) {
        let node = &self.nodes[n as usize];
        out.extend(node.requires.iter().filter(|&&p| !known.contains(p)).map(|&p| Missing::Prerequisite(p)));
        if !node.any_of.is_empty() && !node.any_of.iter().any(|&p| known.contains(p)) {
            out.push(Missing::AnyOf(node.any_of.clone()));
        }
    }

    /// What stands between the tribe and discovering `n` on its own.
    pub fn discovery_gaps(&self, t: &Tribe, n: NodeId, access: &BitSet) -> Vec<Missing> {
        let node = &self.nodes[n as usize];
        let mut out = Vec::new();
        self.prerequisite_gaps(n, &t.known, &mut out);
        if !node.local.is_empty() && !node.local.intersects(t.places) { out.push(Missing::Place(node.local)); }
        out.extend(node.needs.iter().filter(|r| !access.contains(**r)).map(|&r| Missing::Access(r)));
        out
    }

    /// Unknown nodes whose prerequisites are known, with the local gates
    /// still in the way (empty = discoverable now).
    pub fn frontier(&self, t: &Tribe) -> Vec<(NodeId, Vec<Missing>)> {
        let access = self.tribe_access(t);
        (0..self.nodes.len() as NodeId)
            .filter(|&n| !t.known.contains(n) && self.prerequisites_met(n, &t.known))
            .map(|n| (n, self.discovery_gaps(t, n, &access)))
            .collect()
    }

    /// Discovery hazard per year (expected discoveries per year) for `n`;
    /// zero while gated. Over a step of `dt` years the chance is
    /// `1 - exp(-rate * dt)`, so acceleration never saturates.
    pub fn discovery_rate(&self, t: &Tribe, n: NodeId, access: &BitSet, p: DiscoveryParams) -> f32 {
        if t.known.contains(n) || !self.discovery_gaps(t, n, access).is_empty() { return 0.0; }
        let node = &self.nodes[n as usize];
        let people = (t.population as f32 / 30.0).sqrt().clamp(0.2, 4.0);
        let spur = if node.spurs & t.trials != 0 { 2.5 } else { 1.0 };
        let research: f32 = self.unlocks.iter().enumerate()
            .filter(|(i, _)| t.count_of(*i as UnlockId) > 0)
            .flat_map(|(i, u)| u.effects.iter().map(move |e| (i, e)))
            .map(|(i, e)| match e { Effect::Research(x) => x * t.count_of(i as UnlockId) as f32, _ => 0.0 })
            .sum();
        p.base_per_year * p.acceleration.max(0.0) * people * spur * (1.0 + research).min(3.0)
            / (node.effort * (1.0 + 0.35 * node.tier as f32))
    }

    /// Nodes a teacher knows that the learner can take up through contact
    /// (trade, conquest, migration, looting). The local place gate does not
    /// apply; prerequisites do.
    pub fn learnable_from(&self, learner: &BitSet, teacher: &BitSet) -> Vec<NodeId> {
        teacher.iter().filter(|&n| !learner.contains(n) && self.prerequisites_met(n, learner)).collect()
    }

    /// Regional or cultural rendition of an unlock: the first variant whose
    /// places and lean match wins, else the base.
    pub fn rendition(&self, u: UnlockId, t: &Tribe) -> (&str, &str, &[(ResourceId, f32)]) {
        let unlock = &self.unlocks[u as usize];
        let v = unlock.variants.iter().find(|v| {
            (v.when.is_empty() || v.when.intersects(t.places)) && (v.lean.is_none() || v.lean == t.lean)
        });
        match v {
            Some(v) => (&v.name, &v.look, v.cost.as_deref().unwrap_or(&unlock.cost)),
            None => (&unlock.name, &unlock.look, &unlock.cost),
        }
    }

    /// Every unlock of every known node, rendered for this tribe, with what
    /// is still missing to build or make one now.
    pub fn build_menu(&self, t: &Tribe) -> Vec<MenuEntry<'_>> {
        let access = self.tribe_access(t);
        let mut menu = Vec::new();
        for n in t.known.iter() {
            for &u in &self.nodes[n as usize].unlocks {
                let (name, look, cost) = self.rendition(u, t);
                let mut missing = Vec::new();
                for &(r, need) in cost {
                    if !access.contains(r) {
                        missing.push(Missing::Access(r));
                    } else if t.stock_of(r) < need {
                        missing.push(Missing::Stock { resource: r, have: t.stock_of(r), need });
                    }
                }
                for c in &self.unlocks[u as usize].conditions {
                    match *c {
                        Cond::Population(need) if t.population < need => missing.push(Missing::Population { have: t.population, need }),
                        Cond::Has(x) if t.count_of(x) == 0 => missing.push(Missing::Has(x)),
                        Cond::Node(x) if !t.known.contains(x) => missing.push(Missing::Node(x)),
                        Cond::Resource(r) if !access.contains(r) => missing.push(Missing::Access(r)),
                        Cond::Place(p) if !p.intersects(t.places) => missing.push(Missing::Place(p)),
                        Cond::Trial(s) if t.trials & s.bit() == 0 => missing.push(Missing::Trial(s)),
                        _ => {}
                    }
                }
                menu.push(MenuEntry { unlock: u, name, look, cost, missing });
            }
        }
        menu
    }

    /// Derived era label: the one on the highest-tier known node that has one.
    pub fn era<'a>(&'a self, known: &BitSet) -> Option<&'a str> {
        known.iter()
            .filter_map(|n| { let node = &self.nodes[n as usize]; node.era.as_deref().map(|e| (node.tier, e)) })
            .max_by_key(|(tier, _)| *tier)
            .map(|(_, e)| e)
    }

    /// Readable text for a gap, for panels and logs.
    pub fn describe(&self, m: &Missing) -> String {
        let node = |n: NodeId| self.nodes[n as usize].name.as_str();
        let res = |r: ResourceId| self.resources[r as usize].name.as_str();
        match m {
            Missing::Prerequisite(n) | Missing::Node(n) => format!("needs {}", node(*n)),
            Missing::AnyOf(ns) => format!("needs one of {}", ns.iter().map(|&n| node(n)).collect::<Vec<_>>().join(", ")),
            Missing::Place(_) => "not found in this land".into(),
            Missing::Access(r) => format!("no source of {}", res(*r)),
            Missing::Stock { resource, have, need } => format!("{} {:.0}/{:.0}", res(*resource), have, need),
            Missing::Population { have, need } => format!("population {have}/{need}"),
            Missing::Has(u) => format!("needs a {}", self.unlocks[*u as usize].name),
            Missing::Trial(s) => format!("only after surviving {s:?}"),
        }
    }
}
