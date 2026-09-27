# Research Survey: Autonomous Civilization Simulation

## Core conclusion
Use real scientific data and equations as ground truth, but simulate them hierarchically. The game must preserve causal constraints without attempting molecular/continuum simulation everywhere.

## Open-source systems to benchmark
- Mesa: agent scheduling, spaces, reproducible ABM experiments and model analysis.
- FLAME GPU 2: design patterns for very large agent populations and communication/birth/death on GPU.
- Emergent AI (Rust): modular FSM, Utility AI, GOAP, selector/sequencer/parallel composition. Study concepts; do not let a planner become a scripted civilization brain.
- Big Brain / Bevy: scorer/action architecture and parallel utility evaluation. GitHub repo is archived/moved; use as design reference, not a hard dependency until Bevy compatibility is verified.
- Rapier: Rust rigid-body/contact/joint physics; candidate for local high-fidelity interactions.
- Salva: Rust SPH fluid simulation; candidate only for localized experiments, not world-scale hydrology.
- noise-rs: procedural continuous fields for terrain/climate seeds.
- The Powder Toy: benchmark material interaction, heat, pressure, velocity and cellular material rules.
- Thrive: benchmark auto-evolution, species/population simulation and separation of simulation parameters from presentation.
- KeeperRL: inspect creature/monster AI and simulation/game separation; GPL means concepts can be studied but code reuse requires license review.
- Veloren: benchmark large Rust game crate decomposition, procedural world architecture and server/world separation.
- OpenTTD: benchmark long-lived deterministic simulation, pathfinding and compact state.
- FAO AquaCrop: benchmark crop/soil/water abstraction that deliberately balances accuracy, simplicity and robustness.

## Commercial/game design benchmarks
- Dwarf Fortress: generated geography/history/language; explicit historical figures plus abstracted background population is directly relevant to our Agent/Cohort LOD.
- Songs of Syx: very large populations while retaining individual citizens; mundane tasks automated.
- RimWorld: needs, traits, relationships, local pawn stories; do not copy its fixed crafting/tech structure.
- WorldBox: god intervention UX, disasters, civilizations, colonization and world observation.
- Eco: ecosystem consequences, resource extraction feedback and civilization/environment coupling.
- Noita: material identity + simplified chemistry/thermodynamics at pixel scale; benchmark for readable material interactions, not for world-scale simulation.
- The Universim: benchmark god-game interaction and procedural planets, but its fixed research progression is specifically not our target.

## Scientific ground-truth sources
- NIST Periodic Table / physical reference data: atomic properties and constants.
- NIST Chemistry WebBook: thermochemistry and thermophysical fluid properties. SRD licensing must be checked before bundling raw tables.
- NIST Alloy Data / Ceramics Data Portal: metal/alloy and ceramic property references; preserve citations/provenance.
- Materials Project: elastic tensors, bulk/shear moduli, formation energies and related computed properties. API/data terms require attribution and prohibit indiscriminate redistribution.
- IAPWS-95 / IF97: water/steam thermodynamic ground truth; use faster approximations or lookup tables at runtime.
- USGS: open-channel flow, Manning resistance, stream power/erosion, slope stability.
- USDA NRCS: soil texture, water holding and hydraulic-property references.
- FAO Penman-Monteith / AquaCrop: evapotranspiration, crop water stress, biomass/yield abstraction.
- NASA: drag/lift, ideal gas, atmospheric/heat-transfer reference equations.
- IUPAC/NIST: atomic weights and elemental definitions.

## Rule
Every imported dataset must have: source, version/access date, units, uncertainty/range, license/redistribution status, transformation notes and game-level approximation notes.
