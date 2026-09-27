# Hierarchical Physics Kernel

## Fidelity tiers
T0 Macro: region/cohort statistics.
T1 Field: tile/chunk temperature, moisture, flow, soil, biomass, wind.
T2 Object: structures, tools, boats, projectiles using analytic approximations.
T3 Local physics: selected collisions/joints/rigid bodies via Rapier; optional local fluid experiment via Salva.
T4 Offline validation: expensive design evaluation/headless experiments, never every-frame world simulation.

## Mechanics
- Newton: F = m a
- gravity/weight: W = m g
- momentum: p = m v; impulse J = Δp
- kinetic energy: 1/2 m v²
- friction: |F_f| <= μ N
- stress: σ = F/A
- strain: ε = ΔL/L
- elastic region: σ = E ε
- simple bending/beam and Euler buckling approximations for structures
- failure uses yield/ultimate strength + fracture/fatigue modifiers, not a single durability stat

## Fluids and navigation
- hydrostatic pressure: p = p0 + ρgh
- buoyancy: F_b = ρ_fluid g V_displaced
- drag: D = 1/2 ρ v² C_d A
- open-channel macro flow: Manning equation
- mass conservation for storage/flood routing
- local wave/current severity modifies craft stability and steering requirements
- river crossing and open-ocean navigation are distinct capability envelopes

## Thermal
- heat capacity: Q = m c ΔT
- conduction: Fourier form q = -k A ∇T
- convection/radiation represented by effective coefficients at normal gameplay fidelity
- phase transitions consume/release latent heat where materially relevant
- ignition requires temperature + fuel + oxidizer + exposure; no arbitrary fire chance

## Atmosphere/weather
- ideal gas approximation where adequate: pV=nRT
- altitude/pressure/temperature fields are chunk-scale
- humidity, wind, precipitation and evaporation feed soil/crop/fire systems
- no full CFD for global weather

## Hydrology/geology
- rainfall -> interception/infiltration/runoff/storage
- soil texture controls infiltration and water holding
- Manning/open-channel flow for rivers
- stream power/shear proxies for erosion/deposition
- slope failure uses cohesion, friction angle, slope, unit weight and saturation factor-of-safety approximation
- erosion changes terrain slowly and can alter settlement risk over centuries

## Chemistry/material processing
- composition + phase + microstructure/process history determine properties
- Arrhenius-type temperature dependence for selected reaction/degradation rates
- smelting requires heat/reducing conditions/ore chemistry; no named-age unlock
- mixing/alloying/composites derive new property vectors with nonlinear corrections added as validated

## Biology/ecology
- energy/food budgets constrain organisms
- biomass carrying capacity and regeneration
- predator/prey and competition models only as macro approximations; detailed agents use local foraging/reproduction
- disease later: compartmental macro model + individual exposure near focus

## Principle
A higher fidelity tier may correct a lower-tier estimate, but rendering and AI text can never override physical truth.
