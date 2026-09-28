# Automated scenario matrix

Every major simulation change should run deterministic seeds across:
1. river valley / normal resources
2. island / deep-water isolation
3. alpine / cliff isolation
4. no accessible metal
5. scarce wood
6. arid food pressure
7. severe cold
8. repeated flood cycle
9. weak reproducing predators
10. apex monster near early settlement
11. apex monster far from settlement (must not leak damage)
12. multiple civilizations / early contact
13. resource collapse after growth
14. long-run data budget
15. multi-seed anti-convergence

Key invariants:
- no water/cliff traversal without adequate capabilities
- no remote disaster/monster state leakage
- no technology appears solely because year/population crossed a threshold
- extinction is allowed
- failed experiments change evidence, not physical truth
- rendering state never changes simulation truth

Long-run invariants (100-year smoke):
- represented population never drops below simulated alive
- cohort counts stay finite and non-negative
- eras compact instead of discarding (eras grow with runtime)
- civilizations re-derive from lineage/contact, never assigned
- rich preset sustains at least as many alive as arid
