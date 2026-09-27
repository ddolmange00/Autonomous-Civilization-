# Sim Viewer

Developer-facing live observer for sim-core. The viewer must never become authoritative simulation logic.

## Run
```bash
git checkout feat/simulation-foundation-v08
cargo run -p sim-viewer
```

First Bevy build is intentionally much slower than later launches.

## Controls
- 1 / 2 / 3 / 4 / 5: x1 / x5 / x20 / x100 / x1000 simulation speed
- Space: pause
- WASD or arrows: pan camera
- + / -: zoom
- M: spawn a monster
- R: restart with next deterministic seed
- F3: toggle cognition/debug details
- Left click: inspect resident or monster

## What to inspect
A resident panel shows health, current primitive action, needs, personality traits, top action scores and memory count. This is the primary tool for catching irrational behavior.

The initial sandbox is deliberately small: 36 residents, a river barrier, vegetation, rock faces and optional monsters. It exists to validate agency and causal rules before art production.

## Debugging rule
If a resident does something surprising, first inspect:
1. what it could perceive,
2. which affordances were generated,
3. action scores,
4. memory/learned value,
5. needs and personality,
6. actual physical result.

Do not fix surprising behavior by adding scenario scripts unless a missing physical affordance or perception rule is proven.
