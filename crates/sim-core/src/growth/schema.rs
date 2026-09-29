//! Authored form of the growth content (`content/growth/*.ron`).
//! Ids are strings here; `Catalog::compile` resolves them to dense indices.
use serde::Deserialize;

/// Mirrors `terrain::Biome`; `region::biome_tag` maps one onto the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum BiomeTag {
    Ocean, Freshwater, Beach, Ice, Tundra, Taiga, ColdSteppe, Grassland,
    TemperateForest, Swamp, Savanna, Desert, Rainforest, Alpine, Snowcap,
}

impl BiomeTag {
    pub const ALL: [Self; 15] = [
        Self::Ocean, Self::Freshwater, Self::Beach, Self::Ice, Self::Tundra, Self::Taiga,
        Self::ColdSteppe, Self::Grassland, Self::TemperateForest, Self::Swamp, Self::Savanna,
        Self::Desert, Self::Rainforest, Self::Alpine, Self::Snowcap,
    ];
}

/// A property of the land around a tribe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Place { Biome(BiomeTag), Coast, River, Lake, Hills, Mountain }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ResourceCategory { Food, Wood, Stone, Ore, Metal, Fiber, Animal, Mineral, Crafted, Luxury }

/// Where a resource occurs naturally. `chance` is the share of regions with
/// that place that actually hold it, so two mountain tribes can differ.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub place: Place,
    #[serde(default = "one")]
    pub chance: f32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceDef {
    pub id: String,
    pub name: String,
    pub category: ResourceCategory,
    /// Empty: only obtainable by producing it (smelting, weaving, farming).
    #[serde(default)]
    pub sources: Vec<Source>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceFile {
    pub resources: Vec<ResourceDef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Domain {
    Food, ToolsWeapons, Armour, Architecture, Religion, Professions,
    Materials, Transport, Medicine, Governance,
}

impl Domain {
    pub const ALL: [Self; 10] = [
        Self::Food, Self::ToolsWeapons, Self::Armour, Self::Architecture, Self::Religion,
        Self::Professions, Self::Materials, Self::Transport, Self::Medicine, Self::Governance,
    ];
}

/// Hardship that pushes a tribe toward a node (a trial it has survived).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Spur {
    Famine, Raided, Siege, War, Cold, Heat, Flood, Drought, Plague, Monsters,
    Crowding, Trade, Unrest, Death,
}

impl Spur {
    pub const ALL: [Self; 14] = [
        Self::Famine, Self::Raided, Self::Siege, Self::War, Self::Cold, Self::Heat, Self::Flood,
        Self::Drought, Self::Plague, Self::Monsters, Self::Crowding, Self::Trade, Self::Unrest, Self::Death,
    ];
    pub fn bit(self) -> u32 { 1 << self as u32 }
}

/// Dominant cultural lean, matching `culture::EmergentProfile` axes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum CultureLean { Confrontation, Avoidance, Experimentation, Cooperation, Construction }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum UnlockKind { Building, Weapon, Armour, Tool, Dish, Job, Rite, Law, Vessel, Process }

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum Condition {
    Population(u32),
    /// Id of a `Building` unlock the tribe must already have built.
    Building(String),
    /// Id of a `Job` unlock at least one resident must hold.
    Job(String),
    Node(String),
    /// The tribe must have access (local, imported or produced).
    Resource(String),
    /// Any one of these places must be present.
    Place(Vec<Place>),
    Trial(Spur),
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub enum Effect {
    Shelter(f32), Storage(f32), Defense(f32), Attack(f32), Range(f32), Armor(f32),
    FoodYield(f32), Healing(f32), Faith(f32), Loyalty(f32), Research(f32), Speed(f32),
    Capacity(f32), Vision(f32), Fertility(f32), Morale(f32), Trade(f32),
}

/// Regional or cultural rendition of an unlock: same function, own name,
/// look and (optionally) recipe.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VariantDef {
    /// Any one of these places selects the variant; empty = any place.
    #[serde(default)]
    pub when: Vec<Place>,
    #[serde(default)]
    pub lean: Option<CultureLean>,
    pub name: String,
    #[serde(default)]
    pub look: String,
    #[serde(default)]
    pub cost: Option<Vec<(String, f32)>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnlockDef {
    pub id: String,
    pub name: String,
    pub kind: UnlockKind,
    #[serde(default)]
    pub cost: Vec<(String, f32)>,
    /// Person-days of labour to build or make one.
    #[serde(default)]
    pub work: f32,
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    /// Building footprint in tiles.
    #[serde(default)]
    pub footprint: Option<(u8, u8)>,
    /// Per work cycle; a producer makes its outputs accessible to the tribe.
    #[serde(default)]
    pub consumes: Vec<(String, f32)>,
    #[serde(default)]
    pub produces: Vec<(String, f32)>,
    /// Procedural sprite key for the base rendition.
    #[serde(default)]
    pub look: String,
    #[serde(default)]
    pub variants: Vec<VariantDef>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeDef {
    pub id: String,
    pub name: String,
    /// Depth of the node; every prerequisite must have a lower tier.
    pub tier: u8,
    /// All of these must be known.
    #[serde(default)]
    pub requires: Vec<String>,
    /// At least one of these must be known (if any are listed).
    #[serde(default)]
    pub any_of: Vec<String>,
    /// Local discovery needs any one of these places (empty = anywhere).
    /// Learning the node from another people ignores this gate.
    #[serde(default)]
    pub local: Vec<Place>,
    /// Resources the tribe must have access to in order to discover it.
    #[serde(default)]
    pub needs: Vec<String>,
    /// Carried by every founding band (fire, stone tools, foraging...).
    #[serde(default)]
    pub starting: bool,
    /// Relative discovery effort (1 = ordinary).
    #[serde(default = "one")]
    pub effort: f32,
    #[serde(default)]
    pub spurs: Vec<Spur>,
    /// Era label this node earns; the highest-tier one held names the era.
    #[serde(default)]
    pub era: Option<String>,
    #[serde(default)]
    pub unlocks: Vec<UnlockDef>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomainFile {
    pub domain: Domain,
    pub nodes: Vec<NodeDef>,
}

fn one() -> f32 { 1.0 }
