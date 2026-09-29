# Sim Viewer

Developer-facing live observer for sim-core. The viewer must never become authoritative simulation logic.

## Run
```bash
git checkout feat/simulation-foundation-v08
cargo run -p sim-viewer
```

First Bevy build is intentionally much slower than later launches.

## World and presentation
The viewer generates a tile world (`sim-core::terrain`) and founds the band on
its most habitable site. Default: continents, 1024x1024 tiles.

Presentation is isometric (2:1 diamonds). Terrain is baked per 64x64-tile
chunk (`terrain::iso`): every land tile is lifted by its real height with
earth/rock faces toward lower neighbours. A coarse base layer covers the whole
map; detail chunks (8 or 16 px per tile) stream in for what is on screen when
zoomed in. Residents, trees, rocks, houses, building sites and animals are
procedural pixel sprites (`sim-viewer/src/sprites.rs`) drawn from simulation
state: skin from heritable pigmentation, clothes by household, walk cycle
while moving, trees by biome and remaining growth, roofs by material and
integrity.

Physics shown on the map: fire burns tile fuel, spreads with wind and dryness,
stops at water and leaves scorched ground (`terrain::hazards`); floodwater
flows downhill, pools and drains. Residents perceive nearby flames as a heat
source and may flee from or approach them. Smoke, embers, rain, water glints,
earthquake dust and camera shake are particles driven by that state.

UI is Korean, set in the embedded Galmuri14 pixel font (SIL OFL 1.1, see
`crates/sim-viewer/assets/fonts/Galmuri-OFL.txt`). Selection cards show names,
activity in words and bars; raw decision scores appear only with F3.

## Controls
Mouse
- Wheel: zoom toward the cursor
- Right or middle drag: pan; cursor at a window edge: scroll
- Left click: inspect (Inspect tool) or apply the selected god tool
- Left drag with the Inspect tool: box-select residents (group card)
- Village name plates: click to open the village card
- Top bar: pause and speed, map mode, world template, map size, new world, help
- Bottom dock: tool categories (Observe, Life, Nature, Disaster) and tools

Keyboard
- 1 / 2 / 3 / 4 / 5: x1 / x5 / x20 / x100 / x1000 simulation speed
- Space: pause
- WASD or arrows: pan; + / -: zoom
- R: new world with the next seed; `: next template; \: next size; Tab: map mode
- I H Z M T O N X F G Q: god tools; [ ]: radius; , .: power
- L: monster lab; V: settlement pulse; F3: cognition details; F1: shortcut help

## What to inspect
A resident panel shows health, current primitive action, needs, personality traits, top action scores and memory count. This is the primary tool for catching irrational behavior.

The founding band is 36 residents. Vegetation, loose material, rock faces and coastal water markers are sampled from real tiles within 64 tiles of the start site; deep water and peaks block walking (households with working craft may sail).

## Visual capture
`SIM_VIEWER_CAPTURE=<dir> cargo run -p sim-viewer` saves village, whole-map, biome-mode and rebuilt-world screenshots, then exits. `SIM_VIEWER_TEMPLATE=<template>` selects the template.

## Debugging rule
If a resident does something surprising, first inspect:
1. what it could perceive,
2. which affordances were generated,
3. action scores,
4. memory/learned value,
5. needs and personality,
6. actual physical result.

Do not fix surprising behavior by adding scenario scripts unless a missing physical affordance or perception rule is proven.
