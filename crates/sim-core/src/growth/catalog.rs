//! Compiled growth catalog: string ids resolved to dense indices, every
//! reference checked, prerequisite cycles and dead nodes rejected.
use super::schema::*;
use std::collections::HashMap;
use std::fmt;

pub type NodeId = u16;
pub type ResourceId = u16;
pub type UnlockId = u16;

/// Dense bit set over node, resource or unlock indices.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct BitSet(Vec<u64>);

impl BitSet {
    pub fn with_len(n: usize) -> Self { Self(vec![0; n.div_ceil(64)]) }
    pub fn contains(&self, i: u16) -> bool {
        self.0.get(i as usize / 64).is_some_and(|w| w >> (i % 64) & 1 == 1)
    }
    /// Returns true if the bit was newly set.
    pub fn insert(&mut self, i: u16) -> bool {
        let w = i as usize / 64;
        if w >= self.0.len() { self.0.resize(w + 1, 0); }
        let before = self.0[w];
        self.0[w] |= 1 << (i % 64);
        before != self.0[w]
    }
    pub fn remove(&mut self, i: u16) {
        if let Some(w) = self.0.get_mut(i as usize / 64) { *w &= !(1 << (i % 64)); }
    }
    pub fn union_with(&mut self, other: &BitSet) {
        if other.0.len() > self.0.len() { self.0.resize(other.0.len(), 0); }
        for (a, b) in self.0.iter_mut().zip(&other.0) { *a |= b; }
    }
    pub fn len(&self) -> usize { self.0.iter().map(|w| w.count_ones() as usize).sum() }
    pub fn is_empty(&self) -> bool { self.0.iter().all(|w| *w == 0) }
    pub fn iter(&self) -> impl Iterator<Item = u16> + '_ {
        self.0.iter().enumerate().flat_map(|(wi, &w)| {
            (0..64u16).filter(move |b| w >> b & 1 == 1).map(move |b| wi as u16 * 64 + b)
        })
    }
}

/// Set of `Place`s as a bitmask: biomes 0..15, then coast, river, lake, hills, mountain.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PlaceSet(pub u32);

impl PlaceSet {
    pub fn bit(p: Place) -> u32 {
        1 << match p {
            Place::Biome(b) => b as u32,
            Place::Coast => 15,
            Place::River => 16,
            Place::Lake => 17,
            Place::Hills => 18,
            Place::Mountain => 19,
        }
    }
    pub fn of(places: &[Place]) -> Self { Self(places.iter().fold(0, |m, p| m | Self::bit(*p))) }
    pub fn all() -> Self { Self((1 << 20) - 1) }
    pub fn insert(&mut self, p: Place) { self.0 |= Self::bit(p); }
    pub fn contains(self, p: Place) -> bool { self.0 & Self::bit(p) != 0 }
    pub fn intersects(self, other: PlaceSet) -> bool { self.0 & other.0 != 0 }
    pub fn is_empty(self) -> bool { self.0 == 0 }
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub id: String,
    pub name: String,
    pub category: ResourceCategory,
    pub sources: Vec<Source>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Cond {
    Population(u32),
    Has(UnlockId),
    Node(NodeId),
    Resource(ResourceId),
    Place(PlaceSet),
    Trial(Spur),
}

#[derive(Clone, Debug)]
pub struct Variant {
    pub when: PlaceSet,
    pub lean: Option<CultureLean>,
    pub name: String,
    pub look: String,
    pub cost: Option<Vec<(ResourceId, f32)>>,
}

#[derive(Clone, Debug)]
pub struct Unlock {
    pub id: String,
    pub name: String,
    pub kind: UnlockKind,
    pub node: NodeId,
    pub cost: Vec<(ResourceId, f32)>,
    pub work: f32,
    pub conditions: Vec<Cond>,
    pub effects: Vec<Effect>,
    pub footprint: Option<(u8, u8)>,
    pub consumes: Vec<(ResourceId, f32)>,
    pub produces: Vec<(ResourceId, f32)>,
    pub look: String,
    pub variants: Vec<Variant>,
}

#[derive(Clone, Debug)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub domain: Domain,
    pub tier: u8,
    pub requires: Vec<NodeId>,
    pub any_of: Vec<NodeId>,
    /// Empty = discoverable anywhere.
    pub local: PlaceSet,
    pub needs: Vec<ResourceId>,
    pub starting: bool,
    pub effort: f32,
    pub spurs: u32,
    pub era: Option<String>,
    pub unlocks: Vec<UnlockId>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CatalogError {
    Parse { file: String, message: String },
    Duplicate { kind: &'static str, id: String },
    Unknown { kind: &'static str, id: String, referenced_by: String },
    WrongKind { id: String, expected: UnlockKind, referenced_by: String },
    TierOrder { node: String, prerequisite: String },
    BadNumber { at: String, detail: String },
    NoSource { resource: String },
    Unreachable { node: String },
    Unobtainable { resource: String },
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { file, message } => write!(f, "{file}: {message}"),
            Self::Duplicate { kind, id } => write!(f, "duplicate {kind} id `{id}`"),
            Self::Unknown { kind, id, referenced_by } => write!(f, "`{referenced_by}` references unknown {kind} `{id}`"),
            Self::WrongKind { id, expected, referenced_by } => write!(f, "`{referenced_by}` expects `{id}` to be a {expected:?}"),
            Self::TierOrder { node, prerequisite } => write!(f, "`{node}` must have a higher tier than its prerequisite `{prerequisite}`"),
            Self::BadNumber { at, detail } => write!(f, "`{at}`: {detail}"),
            Self::NoSource { resource } => write!(f, "resource `{resource}` has no natural source and no producer"),
            Self::Unreachable { node } => write!(f, "node `{node}` cannot be reached even with every place and resource"),
            Self::Unobtainable { resource } => write!(f, "resource `{resource}` can never be obtained"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Catalog {
    pub resources: Vec<Resource>,
    pub nodes: Vec<Node>,
    pub unlocks: Vec<Unlock>,
    /// Unlocks whose `produces` lists each resource.
    pub producers: Vec<Vec<UnlockId>>,
    resource_index: HashMap<String, ResourceId>,
    node_index: HashMap<String, NodeId>,
    unlock_index: HashMap<String, UnlockId>,
}

fn ron_options() -> ron::Options {
    ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME)
}

impl Catalog {
    /// Parse and compile. `domains` pairs a file label with its text.
    pub fn parse(resources: (&str, &str), domains: &[(&str, &str)]) -> Result<Self, Vec<CatalogError>> {
        let mut errors = Vec::new();
        let opts = ron_options();
        let res: Option<ResourceFile> = opts.from_str(resources.1)
            .map_err(|e| errors.push(CatalogError::Parse { file: resources.0.into(), message: e.to_string() }))
            .ok();
        let mut files = Vec::new();
        for (label, text) in domains {
            match opts.from_str::<DomainFile>(text) {
                Ok(f) => files.push(f),
                Err(e) => errors.push(CatalogError::Parse { file: (*label).into(), message: e.to_string() }),
            }
        }
        match (res, errors.is_empty()) {
            (Some(res), true) => Self::compile(res, files),
            _ => Err(errors),
        }
    }

    pub fn compile(res: ResourceFile, files: Vec<DomainFile>) -> Result<Self, Vec<CatalogError>> {
        let mut errors = Vec::new();
        let mut resource_index = HashMap::new();
        let mut resources = Vec::new();
        for r in res.resources {
            if resource_index.insert(r.id.clone(), resources.len() as ResourceId).is_some() {
                errors.push(CatalogError::Duplicate { kind: "resource", id: r.id.clone() });
            }
            for s in &r.sources {
                if !(s.chance > 0.0 && s.chance <= 1.0) {
                    errors.push(CatalogError::BadNumber { at: r.id.clone(), detail: format!("source chance {} not in (0, 1]", s.chance) });
                }
            }
            resources.push(Resource { id: r.id, name: r.name, category: r.category, sources: r.sources });
        }

        // First pass: ids for nodes and unlocks, so references may point forward.
        let mut node_index = HashMap::new();
        let mut unlock_index: HashMap<String, UnlockId> = HashMap::new();
        let mut unlock_kinds = Vec::new();
        let mut defs = Vec::new();
        for f in files {
            for n in f.nodes {
                if node_index.insert(n.id.clone(), defs.len() as NodeId).is_some() {
                    errors.push(CatalogError::Duplicate { kind: "node", id: n.id.clone() });
                }
                for u in &n.unlocks {
                    if unlock_index.insert(u.id.clone(), unlock_kinds.len() as UnlockId).is_some() {
                        errors.push(CatalogError::Duplicate { kind: "unlock", id: u.id.clone() });
                    }
                    unlock_kinds.push(u.kind);
                }
                defs.push((f.domain, n));
            }
        }

        let res_id = |id: &str, by: &str, errors: &mut Vec<CatalogError>| -> Option<ResourceId> {
            let r = resource_index.get(id).copied();
            if r.is_none() { errors.push(CatalogError::Unknown { kind: "resource", id: id.into(), referenced_by: by.into() }); }
            r
        };
        let node_id = |id: &str, by: &str, errors: &mut Vec<CatalogError>| -> Option<NodeId> {
            let r = node_index.get(id).copied();
            if r.is_none() { errors.push(CatalogError::Unknown { kind: "node", id: id.into(), referenced_by: by.into() }); }
            r
        };
        let amounts = |list: &[(String, f32)], by: &str, errors: &mut Vec<CatalogError>| -> Vec<(ResourceId, f32)> {
            list.iter().filter_map(|(id, q)| {
                if !(q.is_finite() && *q > 0.0) {
                    errors.push(CatalogError::BadNumber { at: by.into(), detail: format!("amount of `{id}` must be positive, got {q}") });
                }
                res_id(id, by, errors).map(|r| (r, *q))
            }).collect()
        };

        let mut nodes = Vec::with_capacity(defs.len());
        let mut unlocks = Vec::with_capacity(unlock_kinds.len());
        for (domain, n) in &defs {
            let nid = nodes.len() as NodeId;
            let requires: Vec<NodeId> = n.requires.iter().filter_map(|r| node_id(r, &n.id, &mut errors)).collect();
            let any_of: Vec<NodeId> = n.any_of.iter().filter_map(|r| node_id(r, &n.id, &mut errors)).collect();
            let needs = n.needs.iter().filter_map(|r| res_id(r, &n.id, &mut errors)).collect();
            if !(n.effort.is_finite() && n.effort > 0.0) {
                errors.push(CatalogError::BadNumber { at: n.id.clone(), detail: format!("effort must be positive, got {}", n.effort) });
            }
            let mut ids = Vec::new();
            for u in &n.unlocks {
                let by = u.id.as_str();
                let conditions = u.conditions.iter().filter_map(|c| match c {
                    Condition::Population(p) => Some(Cond::Population(*p)),
                    Condition::Building(id) | Condition::Job(id) => {
                        let expected = if matches!(c, Condition::Building(_)) { UnlockKind::Building } else { UnlockKind::Job };
                        match unlock_index.get(id) {
                            None => { errors.push(CatalogError::Unknown { kind: "unlock", id: id.clone(), referenced_by: by.into() }); None }
                            Some(&x) if unlock_kinds[x as usize] != expected => {
                                errors.push(CatalogError::WrongKind { id: id.clone(), expected, referenced_by: by.into() }); None
                            }
                            Some(&x) => Some(Cond::Has(x)),
                        }
                    }
                    Condition::Node(id) => node_id(id, by, &mut errors).map(Cond::Node),
                    Condition::Resource(id) => res_id(id, by, &mut errors).map(Cond::Resource),
                    Condition::Place(p) => Some(Cond::Place(PlaceSet::of(p))),
                    Condition::Trial(s) => Some(Cond::Trial(*s)),
                }).collect();
                let variants = u.variants.iter().map(|v| Variant {
                    when: PlaceSet::of(&v.when),
                    lean: v.lean,
                    name: v.name.clone(),
                    look: v.look.clone(),
                    cost: v.cost.as_ref().map(|c| amounts(c, by, &mut errors)),
                }).collect();
                if !(u.work.is_finite() && u.work >= 0.0) {
                    errors.push(CatalogError::BadNumber { at: by.into(), detail: format!("work must be >= 0, got {}", u.work) });
                }
                ids.push(unlocks.len() as UnlockId);
                unlocks.push(Unlock {
                    id: u.id.clone(),
                    name: u.name.clone(),
                    kind: u.kind,
                    node: nid,
                    cost: amounts(&u.cost, by, &mut errors),
                    work: u.work,
                    conditions,
                    effects: u.effects.clone(),
                    footprint: u.footprint,
                    consumes: amounts(&u.consumes, by, &mut errors),
                    produces: amounts(&u.produces, by, &mut errors),
                    look: u.look.clone(),
                    variants,
                });
            }
            nodes.push(Node {
                id: n.id.clone(),
                name: n.name.clone(),
                domain: *domain,
                tier: n.tier,
                requires,
                any_of,
                local: PlaceSet::of(&n.local),
                needs,
                starting: n.starting,
                effort: n.effort,
                spurs: n.spurs.iter().fold(0, |m, s| m | s.bit()),
                era: n.era.clone(),
                unlocks: ids,
            });
        }
        for n in &nodes {
            if n.starting && !(n.requires.is_empty() && n.any_of.is_empty()) {
                errors.push(CatalogError::BadNumber { at: n.id.clone(), detail: "starting nodes cannot have prerequisites".into() });
            }
        }
        // Strictly rising tiers also rule out prerequisite cycles.
        for n in &nodes {
            for &p in n.requires.iter().chain(&n.any_of) {
                if nodes[p as usize].tier >= n.tier {
                    errors.push(CatalogError::TierOrder { node: n.id.clone(), prerequisite: nodes[p as usize].id.clone() });
                }
            }
        }

        let mut producers = vec![Vec::new(); resources.len()];
        for (i, u) in unlocks.iter().enumerate() {
            for (r, _) in &u.produces { producers[*r as usize].push(i as UnlockId); }
        }
        for (i, r) in resources.iter().enumerate() {
            if r.sources.is_empty() && producers[i].is_empty() {
                errors.push(CatalogError::NoSource { resource: r.id.clone() });
            }
        }
        if !errors.is_empty() { return Err(errors); }

        let catalog = Self { resources, nodes, unlocks, producers, resource_index, node_index, unlock_index };
        let (known, access) = catalog.closure(PlaceSet::all(), &catalog.all_natural(), &BitSet::default());
        for (i, n) in catalog.nodes.iter().enumerate() {
            if !known.contains(i as NodeId) { errors.push(CatalogError::Unreachable { node: n.id.clone() }); }
        }
        for (i, r) in catalog.resources.iter().enumerate() {
            if !access.contains(i as ResourceId) { errors.push(CatalogError::Unobtainable { resource: r.id.clone() }); }
        }
        if errors.is_empty() { Ok(catalog) } else { Err(errors) }
    }

    pub fn resource(&self, id: &str) -> Option<ResourceId> { self.resource_index.get(id).copied() }
    pub fn node(&self, id: &str) -> Option<NodeId> { self.node_index.get(id).copied() }
    pub fn unlock(&self, id: &str) -> Option<UnlockId> { self.unlock_index.get(id).copied() }

    /// Every resource that occurs naturally somewhere.
    pub fn all_natural(&self) -> BitSet {
        let mut s = BitSet::with_len(self.resources.len());
        for (i, r) in self.resources.iter().enumerate() {
            if !r.sources.is_empty() { s.insert(i as ResourceId); }
        }
        s
    }

    /// Resources a tribe can obtain: raw access (local or imported) plus
    /// whatever its known producers can make from accessible inputs.
    /// Producer conditions on place and resource apply; population and
    /// building conditions do not (they are attainable, not access).
    pub fn access(&self, known: &BitSet, places: PlaceSet, raw: &BitSet) -> BitSet {
        let mut access = raw.clone();
        loop {
            let mut grew = false;
            for nid in known.iter() {
                for &uid in &self.nodes[nid as usize].unlocks {
                    let u = &self.unlocks[uid as usize];
                    if u.produces.is_empty() || !self.producer_ready(u, places, &access) { continue; }
                    for (r, _) in &u.produces { grew |= access.insert(*r); }
                }
            }
            if !grew { return access; }
        }
    }

    fn producer_ready(&self, u: &Unlock, places: PlaceSet, access: &BitSet) -> bool {
        u.consumes.iter().all(|(r, _)| access.contains(*r))
            && u.conditions.iter().all(|c| match c {
                Cond::Place(p) => p.intersects(places),
                Cond::Resource(r) => access.contains(*r),
                _ => true,
            })
    }

    /// True when prerequisites are known (ignores place and resource gates).
    pub fn prerequisites_met(&self, n: NodeId, known: &BitSet) -> bool {
        let node = &self.nodes[n as usize];
        node.requires.iter().all(|&p| known.contains(p))
            && (node.any_of.is_empty() || node.any_of.iter().any(|&p| known.contains(p)))
    }

    /// Everything reachable by local discovery alone, starting from `known`.
    /// Returns (known nodes, accessible resources).
    pub fn closure(&self, places: PlaceSet, raw: &BitSet, known: &BitSet) -> (BitSet, BitSet) {
        let mut known = known.clone();
        loop {
            let access = self.access(&known, places, raw);
            let mut grew = false;
            for i in 0..self.nodes.len() as NodeId {
                if known.contains(i) { continue; }
                let n = &self.nodes[i as usize];
                if (n.local.is_empty() || n.local.intersects(places))
                    && n.needs.iter().all(|r| access.contains(*r))
                    && self.prerequisites_met(i, &known)
                {
                    grew |= known.insert(i);
                }
            }
            if !grew { return (known, access); }
        }
    }
}
