use sim_core::{affordances::FeatureKind,diplomacy::mean_trust,language::Lexicon,lod::represented_population,sandbox::{Preset,Sandbox}};

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
    causal_total:u64,
    culture_divergence:f32,
    mean_structure_integrity:f32,
    vegetation_remaining:f32,
    mineral_remaining:f32,
    mean_project_work:f32,
    mean_project_material:f32,
    mean_project_age:f32,
    trades:u64,
    thefts:u64,
    contacts:usize,
    mean_trust:f32,
    lexicon_divergence:f32,
    institutions:usize,
    narratives:usize,
    warbands:usize,
    battles:u64,
    repelled:u64,
    drafts:u64,
    civilizations:usize,
    largest_civ:usize,
    represented:f64,
    cohorts:f64,
    eras:usize,
}

fn culture_divergence(sim:&Sandbox)->f32 {
    let active:Vec<_>=sim.settlements.iter().filter(|s|!s.members.is_empty()).collect();
    if active.len()<2{return 0.0;}
    let mut sum=0.0;let mut pairs=0u32;
    for i in 0..active.len(){for j in (i+1)..active.len(){
        let a=active[i].profile();let b=active[j].profile();
        sum+=(a.confrontation-b.confrontation).abs()+(a.avoidance-b.avoidance).abs()+
            (a.experimentation-b.experimentation).abs()+(a.cooperation-b.cooperation).abs()+
            (a.construction-b.construction).abs();
        pairs+=1;
    }}
    sum/(pairs.max(1) as f32*5.0)
}

fn lexicon_divergence(sim:&Sandbox)->f32 {
    let active:Vec<_>=sim.settlements.iter().filter(|s|!s.members.is_empty()).collect();
    if active.len()<2{return 0.0;}
    let mut sum=0.0;let mut pairs=0u32;
    for i in 0..active.len(){for j in (i+1)..active.len(){
        sum+=Lexicon::divergence(&active[i].lexicon,&active[j].lexicon);
        pairs+=1;
    }}
    sum/pairs.max(1) as f32
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
        causal_total:sim.causal_log.total_written,
        culture_divergence:culture_divergence(sim),
        mean_structure_integrity:if sim.structures.is_empty(){0.0}else{sim.structures.iter().map(|s|s.integrity).sum::<f32>()/sim.structures.len() as f32},
        vegetation_remaining:{
            let v:Vec<_>=sim.features.iter().filter(|f|f.kind==FeatureKind::Vegetation).collect();
            if v.is_empty(){0.0}else{v.iter().map(|f|(f.quantity/f.capacity.max(0.001)).clamp(0.0,1.0)).sum::<f32>()/v.len() as f32}
        },
        mineral_remaining:{
            let v:Vec<_>=sim.features.iter().filter(|f|f.kind==FeatureKind::LooseMaterial).collect();
            if v.is_empty(){0.0}else{v.iter().map(|f|(f.quantity/f.capacity.max(0.001)).clamp(0.0,1.0)).sum::<f32>()/v.len() as f32}
        },
        mean_project_work:if sim.projects.is_empty(){0.0}else{sim.projects.iter().map(|p|(p.progress/p.required_work.max(0.001)).clamp(0.0,1.0)).sum::<f32>()/sim.projects.len() as f32},
        mean_project_material:if sim.projects.is_empty(){0.0}else{sim.projects.iter().map(|p|(p.material_committed/p.material_required.max(0.001)).clamp(0.0,1.0)).sum::<f32>()/sim.projects.len() as f32},
        mean_project_age:if sim.projects.is_empty(){0.0}else{sim.projects.iter().map(|p|(sim.year-p.started_year).max(0.0) as f32).sum::<f32>()/sim.projects.len() as f32},
        trades:sim.trade_count,
        thefts:sim.theft_count,
        contacts:sim.contacts.len(),
        mean_trust:mean_trust(&sim.contacts),
        lexicon_divergence:lexicon_divergence(sim),
        institutions:sim.institutions.len(),
        narratives:sim.narratives.len(),
        warbands:sim.warbands.len(),
        battles:sim.battles,
        repelled:sim.monsters_repelled,
        drafts:sim.draft_count,
        civilizations:sim.civilizations.len(),
        largest_civ:sim.civilizations.iter().map(|c|c.settlement_ids.len()).max().unwrap_or(0),
        represented:represented_population(sim.residents.iter().filter(|r|r.health>0.0).count(),&sim.cohorts),
        cohorts:sim.cohorts.iter().map(|c|c.count as f64).sum(),
        eras:sim.eras.len(),
    }
}

fn run_emergence(seed:u64,years:f64,step_days:f32,preset:Preset)->EmergenceSummary {
    let mut sim=Sandbox::with_preset(seed,preset);
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
    let preset=match args.get(5).map(|s|s.as_str()) {
        Some("arid")=>Preset::Arid,
        Some("rich")=>Preset::Rich,
        _=>Preset::Default,
    };

    println!("seed\tyear\talive\ttotal_residents\thouseholds\tactive_settlements\tsettlement_splits\tstructures\tprojects\tmax_design_gen\tknowledge_items\tcausal_records\tcausal_total\tculture_divergence\tmean_structure_integrity\tvegetation_remaining\tmineral_remaining\tmean_project_work\tmean_project_material\tmean_project_age\ttrades\tthefts\tcontacts\tmean_trust\tlexicon_divergence\tinstitutions\tnarratives\twarbands\tbattles\trepelled\tdrafts\tcivilizations\tlargest_civ\trepresented\tcohorts\teras");
    for i in 0..seed_count {
        let s=run_emergence(base_seed+i,years,step_days,preset);
        println!("{}\t{:.1}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{:.0}\t{:.0}\t{}",
            s.seed,s.year,s.alive,s.residents_total,s.households,s.active_settlements,s.settlement_lineages,
            s.structures,s.projects,s.max_design_generation,s.knowledge_items,s.causal_records,s.causal_total,s.culture_divergence,s.mean_structure_integrity,s.vegetation_remaining,s.mineral_remaining,s.mean_project_work,s.mean_project_material,s.mean_project_age,
            s.trades,s.thefts,s.contacts,s.mean_trust,s.lexicon_divergence,s.institutions,s.narratives,
            s.warbands,s.battles,s.repelled,s.drafts,
            s.civilizations,s.largest_civ,s.represented,s.cohorts,s.eras);
    }
}
