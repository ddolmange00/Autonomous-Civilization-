use sim_core::sandbox::Sandbox;

#[derive(Debug)]
struct EmergenceSummary {
    seed:u64,
    year:f64,
    alive:usize,
    residents_total:usize,
    households:usize,
    active_settlements:usize,
    settlement_lineages:usize,
    structures:usize,
    projects:usize,
    max_design_generation:u32,
    knowledge_items:usize,
    causal_records:usize,
}

fn summarize(sim:&Sandbox)->EmergenceSummary {
    EmergenceSummary{
        seed:sim.seed,
        year:sim.year,
        alive:sim.residents.iter().filter(|r|r.health>0.0).count(),
        residents_total:sim.residents.len(),
        households:sim.households.iter().filter(|h|!h.members.is_empty()).count(),
        active_settlements:sim.settlements.iter().filter(|s|!s.members.is_empty()).count(),
        settlement_lineages:sim.settlements.iter().filter(|s|s.parent_id.is_some()).count(),
        structures:sim.structures.len(),
        projects:sim.projects.len(),
        max_design_generation:sim.structures.iter().map(|s|s.design.generation).max().unwrap_or(0),
        knowledge_items:sim.residents.iter().filter(|r|r.health>0.0).map(|r|r.knowledge.items.len()).sum(),
        causal_records:sim.causal_log.nodes.len(),
    }
}

fn run_emergence(seed:u64,years:f64,step_days:f32)->EmergenceSummary {
    let mut sim=Sandbox::new(seed);
    let target=sim.year+years;
    while sim.year<target {
        sim.step(step_days.min(((target-sim.year)*365.0) as f32).max(0.05));
        if sim.residents.iter().all(|r|r.health<=0.0){break;}
    }
    summarize(&sim)
}

fn main() {
    let args:Vec<String>=std::env::args().collect();
    let years=args.get(1).and_then(|x|x.parse::<f64>().ok()).unwrap_or(60.0);
    let seed_count=args.get(2).and_then(|x|x.parse::<u64>().ok()).unwrap_or(5).clamp(1,64);
    let base_seed=args.get(3).and_then(|x|x.parse::<u64>().ok()).unwrap_or(847_291);
    let step_days=args.get(4).and_then(|x|x.parse::<f32>().ok()).unwrap_or(2.0).clamp(0.25,10.0);

    println!("seed\tyear\talive\ttotal_residents\thouseholds\tactive_settlements\tsettlement_splits\tstructures\tprojects\tmax_design_gen\tknowledge_items\tcausal_records");
    for i in 0..seed_count {
        let s=run_emergence(base_seed+i,years,step_days);
        println!("{}\t{:.1}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            s.seed,s.year,s.alive,s.residents_total,s.households,s.active_settlements,s.settlement_lineages,
            s.structures,s.projects,s.max_design_generation,s.knowledge_items,s.causal_records);
    }
}
