# WorldBox Feature Matrix -> Autonomous Civilization

Purpose: benchmark publicly documented gameplay concepts and interaction patterns. Do not copy proprietary code, art, text, names, balance tables or hidden implementation.

## A. World / terrain
Benchmark:
- editable terrain and biomes
- procedural world generation
- islands/continents/oceans
- temperature/environment manipulation
- civilization colonization across geography

Our extension:
- hydrology/soil/climate are causal fields
- deep water/cliffs are physical barriers
- erosion, floodplain fertility and slope failure
- discovery fog is resident/civilization knowledge, not only player fog
- procedural resources have composition/quality, not just resource type

## B. God powers / intervention
Benchmark:
- creature/population spawning
- terrain and biome brushes
- destructive powers/disasters
- environmental changes
- fast sandbox experimentation

Our extension:
- interventions alter world truth only
- no technology/culture outcome button
- radius/intensity/duration/position are explicit
- event propagation is local
- resident awareness and reaction can be inspected after intervention
- every intervention can be replayed under the same seed for debugging

## C. Creatures / animals / monsters
Benchmark:
- many creature types
- traits/status effects
- predators and hostile creatures
- supernatural/extreme entities as sandbox pressure

Our extension:
- hunger, territory, prey availability, reproduction, injury, perception
- monster does not automatically attack civilization
- residents do not automatically know a monster exists
- combat/avoidance/coexistence/migration/extinction emerge
- creature materials can enter civilization material knowledge after contact

## D. Residents / families
Benchmark:
- individual traits and needs
- families and ancestry
- notable individuals
- inspectable characters

Our extension:
- continuous personality rather than trait-only tags
- episodic memory
- imperfect perception
- relationships, trust, prestige, obligations
- individual action scoring and causal debug trace
- promotion from cohort to persistent individual when historically significant

## E. Biological variation / populations
Benchmark:
- species/subspecies/genes and visible population variation

Our extension:
- continuous heritable physical traits
- migration, admixture and local environmental selection
- no ethnicity/race -> intelligence/aggression/morality stat mapping
- cultural identity and biological ancestry are separate systems

## F. Civilization / settlement
Benchmark:
- villages, kingdoms, expansion, colonization
- armies and wars
- diplomacy, alliances and rebellion
- sailing and overseas expansion

Our extension:
- settlement is not assigned a strategy archetype
- institutions emerge from repeated coordination patterns
- physical transport capability constrains expansion
- logistics/food/water/material dependencies matter
- settlement pulse exposes current awareness/action distribution
- civilizations can fragment culturally before political separation

## G. Culture / language / religion
Benchmark:
- cultures
- languages
- religions
- families/clans and social meta-objects

Our extension:
- culture is derived from observed successful/prestigious behavior
- language spreads/mutates through communication networks
- religion can emerge from causal attribution, ritual repetition, charismatic individuals, disasters and transmitted narratives
- multiple local cultures/languages/beliefs may coexist in one political entity
- no fixed historical religion/culture tech path

## H. Knowledge / technology
Benchmark:
- civilization progression and unlockable capabilities

Our deliberate departure (revised 2026-09-29, see `PC_WORLDBOX_VISION.md`):
- authored growth trees per domain, with regional variants and exclusive branches instead of one global tree; era names are derived labels, not gates
- nodes are reached through problem -> observation -> candidate action/design -> physical test -> memory/social transmission
- material/process/design lineage
- copying, theft, trade and convergent discovery
- knowledge can be lost when carriers die or institutions collapse

## I. Warfare
Benchmark:
- armies, invasions, kingdom conflict, walls

Our extension:
- weapons are Design Genomes, not fixed tier unlocks
- wall-like structures are emergent geometry/function combinations
- morale, information, logistics, terrain, weapon physics and individual decisions
- surrender, flight, hiding, raiding and non-participation are possible without scripted war stages

## J. Disasters
Benchmark:
- fire, earthquakes and other sandbox catastrophes

Our extension:
- disasters use environmental fields/physics approximations
- residents perceive locally and may misunderstand causes
- repeated disasters can change settlement location, construction practice, stories and institutions
- recovery is not guaranteed

## K. Observation UI
Benchmark:
- inspectable units/kingdoms/meta-objects
- graphs/maps/statistics

Our extension:
- World -> Settlement -> Resident -> Artifact causal drill-down
- awareness percentage and action distribution
- "why?" trace for resident decisions
- true property vs believed property only in debug mode
- event -> perception -> decision -> outcome -> transmission trace
- information density adapts to zoom

## L. Time / simulation
Benchmark:
- fast-forward sandbox observation

Our extension:
- simulation LOD: agent -> household/cohort -> settlement/region
- same causal constraints at all LODs
- deterministic seed replay
- x1 to headless high-speed scenario mode
- data retention/aggregation budgets

## Priority
P0: God dock + situation awareness + causal inspector + creature/terrain spawning.
P1: families/relationships, animals/ecology, settlement identity, migration/contact, disaster tools.
P2: language/culture/religion emergence, diplomacy/institutions, warfare/logistics, deeper biology.
P3: long-history analytics, advanced machines/energy, disease, richer procedural pixel grammar.
