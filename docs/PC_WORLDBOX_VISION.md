# PC WorldBox-class Vision

Product spec for the PC (and later mobile) game. Extends `GAME_IDENTITY.md`
and keeps its non-negotiables. Owner decisions of 2026-09-29 (revision 2)
supersede three earlier choices: "no tech tree" (now: authored, regional
growth trees), "choose one people" (now: register and manage any number of
tribes), and "Minecraft block building" (now: pick-and-place building
blueprints). Genre research: `database/sources/inbox/2026-09-29-godsim-genre-research.md`.

## One line
You own a living world. It runs by itself. Tribes emerge on their own; you
register the ones you care about and raise, rush, punish, enslave, pit
against each other or abandon them as you like. Left alone they still grow.
Helped, they can do ten years of progress in one and outgrow their
neighbours.

## What each reference game contributes
| Reference | We take | We do not take |
|---|---|---|
| WorldBox | Autonomous world, god powers (spawn, smite, pick up and throw, disasters, terrain brushes), watching kingdoms rise | Stick-figure readability, passive war AI, one kingdom snowballing the map |
| RimWorld | Typed resources and stockpiles, storyteller-driven events, resident traits/mood/relationships, readable sprites and clean info panels | Top-down projection, single-colony scope |
| StarCraft | Camera feel, drag-select, army orders for tribes you manage | Mandatory micro; units that obey unconditionally |
| Clash of Clans | Base defense layout (walls, towers, traps, guardians) tested by real raids | Timers and paywalls |
| Minecraft | Mining, quarrying, night danger, the feeling that the world is made of stuff | Block-by-block editing (tedious in 2.5D) |
| Civilization-style games | Growth trees with visible requirements and unlock lists | Global, identical tree for everyone |

## Player role: four optional layers
The world must advance with zero intervention (acceptance: long
no-intervention runs across many seeds show growth in most seeds and diverse
end states, measured, not asserted). Every layer below is optional.

1. **Watch**: pan, zoom, click anything, open histories.
2. **Place**: choose a building blueprint from the tribe's unlocked list and
   put it on the map; residents gather materials and build it over time.
3. **Command**: declare war, pick targets, rally armies, set growth focus
   (food, war, trade, faith, craft), open or block trade routes.
4. **God powers**: the WorldBox set (see below).

## Tribes: emergence, registration, management
- When residents coalesce into a band, tribe or state, a notification fires
  with its location and origin.
- Click it, name it, and **register** it. Registered tribes appear in the
  **management list**; any tribe in the world can be registered. There is
  no fixed "player tribe".
- Each registered tribe has its own **policy**, independent of the others:
  nurture, harsh (high-pressure storyteller), neglect (no player events),
  or enslaved (see below).
- Unregistered tribes are fully simulated and are the "neighbours" a
  registered tribe competes with.

## Commands, loyalty and punishment
- Control is hybrid (owner decision, 2026-09-29): civilians stay autonomous
  and are steered only through placement, focus and god powers; armies and
  heroes can be drag-selected and given right-click orders (move, attack,
  hold, guard), which they may refuse.
- Commands pass through each resident's decision loop (`guidance.rs`
  signal weighted by trust and conformity). Obedience depends on loyalty,
  fear, and conflict with the resident's own urgent needs.
- Low loyalty means occasional **refusal**. A refusal is shown with its
  reason (hungry, afraid, resentful, loyal to a rival leader).
- The player may answer refusal with force: **lightning**, **pick up and
  throw**, **kill**. Force raises fear and short-term obedience for the
  target and witnesses, and raises resentment. Resentment is the same
  quantity that drives rebellion, so rule by terror has a price.
- Rewards (blessings, feasts, relics) raise loyalty the slow way.

## Slavery and rebellion
- A tribe can be conquered into, or ruled as, an enslaved population:
  forced labour, no say in commands, low loyalty by construction.
- Resentment, numbers, weapons access, a charismatic leader and a weak
  garrison combine into rebellion risk. Rebellions can fail, split the
  state or overthrow it and found a new tribe (which fires a notification).
- Enslaved peoples keep their own knowledge; masters can absorb it, and
  rebels take it with them.

## Growth trees
### Shape
- Domains, each its own tree: **food and farming**, **tools and weapons**,
  **armour**, **architecture and defense**, **religion and rites**,
  **professions**, **materials and processing**, **transport and
  seafaring**, **medicine**, **governance and law**. More may be added.
- A **node** lists what it unlocks (buildings, items, weapons, dishes,
  jobs, rites, laws) and what it requires: prerequisite nodes, materials
  (local or imported), population or specialists, existing structures,
  and situational triggers (e.g. a lost siege, a famine).
- Reaching a node shows the unlock list with each item's material cost and
  conditions. When the conditions are met the item can be built or made.
- Content is authored, dense and precise: many nodes and many items per
  node. It lives in data files, not code, with a validator that checks
  references and reports nodes unreachable from every region.
  Implemented: `crates/sim-core/content/growth/*.ron` (resources plus one
  file per domain, embedded at build time), compiled and validated by
  `sim_core::growth::Catalog` (unknown ids, wrong unlock kinds, tier order,
  dead nodes, unobtainable resources); `tests/growth_catalog.rs` checks that
  no single land reaches everything and that trade widens trees on generated
  worlds. Wired into the sandbox by `growth::runtime`: each settlement is a
  tribe whose known nodes are the `growth::*` keys its living members carry
  (lost with the last carrier); founding bands carry the `starting` nodes;
  discovery, trade imports and trade learning, trial spurs and per-tribe
  acceleration run every 0.25 years. Tribes do not yet build unlocks (needs
  typed stockpiles, phase 6). Discovery pacing (base 0.03/yr) is provisional
  until populations stay viable over century runs.

### Regional character
- Trees differ by biome, local materials, fauna, climate and culture.
  Desert, tundra, coast, forest and mountain peoples have exclusive
  branches and their own variants of shared nodes (name, look, stats,
  recipe). The same "watchtower" node yields a mud tower in one place and
  a timber stilt tower in another.
- Visual variation of weapons, armour and buildings is driven by the
  tribe's materials and culture profile (the design genome idea is kept
  for appearance and stat variation inside a node).

### How a tribe reaches nodes
- Knowledge is carried by residents, not by a global research bar. It is
  gained by pressure and experiment (existing `invention`/`knowledge`
  pipeline), and can be lost when its carriers die or institutions fall.
- **Trade** imports the materials and knowledge a region cannot produce,
  opening branches that were closed locally. Other routes follow the same
  rule: **conquest and enslavement**, **migration and intermarriage**,
  **looting and reverse engineering**.
- Early worlds show sharply distinct regional civilizations; contact mixes
  and advances them; isolated tribes stay distinctive but fall behind.
  Opening or blockading trade routes is a strategy.
- Era names ("stone-using", "bronze-casting", "gunpowder", "electric") are
  labels derived from nodes held, never gates.

## Buildings and placement
- The build menu shows only what the tribe has unlocked, grouped by domain,
  with costs and unmet conditions greyed out with the reason.
- Placing a blueprint creates a construction site. Residents haul materials
  and build in visible stages. Missing materials stall the site and create
  a demand the tribe tries to meet (gather, trade, raid).
- Autonomous tribes use the same catalog through their own AI, so a
  neglected tribe still builds farms, walls and towers when it needs them.
- Defensive set: walls, gates, watchtowers, crossbow towers, turrets (by
  material level), traps. Layout is tested by real raids.

## Acceleration ("ten years in one")
- Acceleration is **per tribe**, not a global fast-forward (global speed
  control stays separate). It widens the gap between a tribe and its
  neighbours.
- Levers: research/discovery rate, construction speed, population growth,
  food output, war preparation.
- Cost: the god's resource (belief), which accumulates from registered
  tribes' faith and prosperity.

## God powers (WorldBox set)
- Spawn animals and creatures; release animals into a tribe's land.
- Place guardian monsters at gates or borders.
- Lightning, pick up and throw residents or creatures, kill.
- Disasters (fire, flood, plague, meteor, earthquake) and blessings
  (fertility, insight, courage).
- Terrain brushes: raise/lower land, water, mountains, forest, ore.
- Visions and commandments (a hint or rule the tribe may misread or reject).

## RimWorld layer
- **Resources**: typed materials (wood, stone, ores, metals, cloth, food
  kinds, components) held in stockpiles and carried by residents.
- **Storyteller**: an event director per tribe, with a difficulty set by the
  tribe's policy. Events: raids, plague, famine, eclipse, cold snap,
  traders, wanderers joining, animal frenzy, monster nests, refugees.
- **Residents**: every resident is fully simulated (traits, needs, mood,
  skills, relationships, memory). The default UI shows tribe-level
  information; clicking a resident opens the full detail panel.
- **Favourites and life stories**: star any resident for a life log
  (birth, family, skills, inventions, trials, battles, death), a portrait
  from heritable traits, event notifications and a follow camera.

## War and defense
- Raids (take and flee) and invasions (hold land, migrate) are distinct
  outcomes, driven by need, grievance and logistics reach. Monster nests
  raid too.
- Clash of Clans loop (owner decision, 2026-09-29): lay out base defenses,
  get raided by AI tribes and monsters, raid them back, and turn loot and
  survival into growth and rewards.
- Managed tribes accept army orders (subject to loyalty). Warfare stays
  logistics-bound, and fission/cohesion limits one state swallowing the map.
- Surviving a trial records the failure mode and raises pressure to solve
  it; growth is likely, not guaranteed; collapse stays possible.

## Goals
Endless sandbox by default. Optional achievements to chase, e.g. "stone age
to spaceflight", "a slave revolt that wins", "three tribes at war at once",
"a tribe that survives a harsh storyteller for 500 years".

## Presentation
- **2.5D, isometric** (owner decision, 2026-09-29): Clash of Clans /
  StarCraft 1 style diamond tiles, terrain with visible height, elevation
  shading and cliffs; buildings and residents drawn with depth sorting.
- **Art is procedurally generated pixel art** (owner decision,
  2026-09-29): residents, buildings, weapons and creatures are composed
  from generated pixel layers, not hand-drawn asset packs.
- **Readable residents**: layered sprites (body, head, clothing, carried
  tool or weapon) so a resident reads as a person at normal zoom, with
  animation for walking, working, fighting and falling.
- **Physical feedback**: projectile arcs, thrown bodies that fall and land,
  fire spread, water flow, structures that visibly take damage and collapse.
- **Game UI**: build menu, god-power bar, tribe management list, tribe and
  resident panels, notification feed. Designed for mouse and touch from
  the start (PC first, mobile later).
- **Seeing creativity**: legends/chronicle browser over the causal log,
  notable-events feed for rare or first-time events, map modes (terrain,
  height, temperature, moisture, biome, then tribe, culture, language,
  religion, trade), procedural heraldry and language-derived names.

## World layer (phase 1)
- Tile map (`sim-core::terrain`), 4 world units per tile, chunked render.
- Sizes 256² to 2048²; templates: continents, archipelago, pangaea,
  islands, lakes.
- Height from seeded gradient noise with domain warp; sea level set by
  land-fraction percentile; temperature from latitude and lapse rate;
  moisture from noise and distance to water; rivers from depression-filled
  flow accumulation; Whittaker-style biome classification.
- The sandbox stands on the map: start site chosen by habitability,
  features (vegetation, loose material, rock, coastal water) are sampled
  from real tiles, and walking is blocked by deep water and peaks.

## Platforms
1. Single-player on the owner's own worlds until quality is high.
2. Release on Steam and Google Play.
3. Online play (versus) only if reception warrants it.

## Phase order
1. World layer, chunked terrain renderer, map modes, camera.
2. 2.5D presentation: isometric height rendering, readable procedural layered residents,
   building construction stages, game UI shell for mouse and touch.
3. Scale: spatial index, no quadratic loops, chunk LOD.
4. Tribe emergence notifications, registration, management list, per-tribe
   policy; commands with loyalty and refusal; lightning, pick up and throw,
   spawn animals.
5. Growth-tree content system (data files, validator, regional variants),
   build menu, blueprint placement, construction.
6. Typed resources and stockpiles, per-tribe storyteller events,
   per-tribe acceleration with belief.
7. Branch spread through trade, conquest, migration and looting; slavery
   and rebellion.
8. Defense structures, guardian monsters, taming, raids and invasions, army
   orders.
9. Chronicle/legends browser, favourites and life stories, achievements.
10. Release builds for Steam and Google Play.
