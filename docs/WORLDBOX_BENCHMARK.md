# WorldBox Benchmark — Adopt the Sandbox, Improve Legibility

## What to benchmark
WorldBox's strongest reference value for this project is interaction design:
- immediate god-power intervention
- terrain/world editing
- spawning creatures and civilizations
- disasters and destructive experiments
- individuals with traits/needs
- civilizations, kingdoms, colonization, sailing, diplomacy, rebellion and war
- fast time controls and observation-first play

We should benchmark the low-friction loop: create -> perturb -> observe -> accelerate time -> inspect outcome.

## Where our project should deliberately differ

### 1. Civilization legibility
The player should be able to answer:
- What is this settlement afraid of right now?
- What problems are residents repeatedly encountering?
- What are residents trying, and why?
- Which behaviors are becoming cultural norms?
- What materials/principles does this group believe it understands?
- Which beliefs are wrong?
- What caused population growth/decline/migration?
- How did this design lineage evolve?

Do not reduce this to one civilization personality label.

### 2. Situation awareness
Events must enter the world through perception.
A monster can exist nearby without every resident instantly knowing it exists.
Knowledge spreads through sight, sound, survivors, communication and social networks.
This creates delayed reaction, misinformation, panic, local heroism and subgroup disagreement naturally.

### 3. Diversity without fixed fantasy races
Do not begin with Human/Elf/Orc/Dwarf gameplay classes.
World generation may create populations with visible biological variation and heritable continuous traits. Culture, technology, clothing, architecture and social identity then diverge through environment/history.
Avoid mapping real human ethnic groups to gameplay stat bonuses or deterministic behavioral traits.

### 4. Readable graphics
Pixel art must prioritize silhouette and semantic readability:
- terrain palette has strong water/land/cliff/forest separation
- residents remain visible against terrain
- selected settlement/agent gets a subtle outline, not visual noise
- buildings show function through shape
- civilization identity uses restrained accents/patterns
- zoom LOD reduces clutter rather than merely shrinking sprites

### 5. Reaction trace
Every important event should support a causal trace:
event -> who perceived it -> belief/memory changes -> decisions -> actions -> physical outcomes -> social transmission -> emergent norm.

## God tools we should eventually expose
Terrain brush; water; vegetation; resource deposit; resident/population spawn; animal/monster spawn; fire; rain; drought; flood; wind/storm; earthquake/landslide; disease later; temperature; material drop; observation marker; player guidance.

God tools modify world truth. They never directly assign a technology or cultural response.

## Context UI
World remains nearly full screen.
Clicking an entity opens contextual information:
- Resident: needs, traits, relationships, memories, perceptions, top action scores, current action
- Settlement: population, food/water, local risks, active problems, recent experiments, norms, knowledge confidence
- Civilization: settlements, migration, contacts, design lineages, cultural divergence, resource dependencies
- Monster/animal: needs, territory, prey, perceived threats, current behavior
- Terrain: soil/water/climate/material/ecology
- Artifact: genome, materials, process history, predicted vs actual performance, lineage

## Core differentiator
WorldBox-like freedom of intervention + Dwarf-Fortress-like causal history + our resident-level emergent agency and physics/material ground truth.
