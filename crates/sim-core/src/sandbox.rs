use crate::{
    affordances::{generate, FeatureKind, LocalCapabilities, PerceivedFeature},
    agency::{ActionPrimitive, AgentMind, Needs, Traits},
    memory::{Episode, EpisodicMemory},
    awareness::{Awareness, SituationKind, SituationReport},
    events::{WorldEvent, WorldEventKind},
    species::{AnimalArchetype, MonsterArchetype},
    blueprints::{MonsterBlueprint,PixelSkin},
    pixel_animation::MotionState,
    knowledge::KnowledgeStore,
    family::Kinship,
    relationships::SocialMemory,
    life_history::{LifeHistory,LifeStage,Sex},
    social_dynamics::{spend_time,partnership_affinity},
    generation::{annual_mortality_risk,conception_propensity,ReproductionContext},
    social_affordances::{generate_social,SocialTarget},
    development::{capability_factor,filter_affordances,mobility_factor},
    demography::{HeritableTraits,inherit},
    households::Household,
    life_history::inherit_personality,
    specialization::PracticeProfile,
    settlement_detection::detect_settlements,
    settlement_identity::{cluster_member_ids,SettlementIdentity},
    culture::CulturalField,
    built_environment::{BuiltStructure,ConstructionProject,evolve_shelter_design,integrity_from,proposal_strength,seed_shelter_design,seed_wall_design,work_value},
    causal_log::{CausalLog, CausalNode},
    world::Position,
    language::Lexicon,
    diplomacy::{contact_between, Contact},
    exchange::{exchange_amount, surplus, TradeRecord},
    institutions::{coordination_bonus, update_institutions, Institution},
    narrative::{misattribution, reinforce, ritual_relief, Narrative},
    warfare::{assemble_force, disbanded, form_warbands, melee_from_repertoire, resolve_battle, update_morale, Warband},
    logistics::{carry_capacity, supply_drain},
    seafaring::{can_embark, craft_from_designs, crosses_water, Craft},
    invention::{seed_float_design, seed_tool_design, test_prototype},
    ground_truth::{canonical_registry, GroundTruthRegistry},
    design::{DesignGenome, Function},
};

#[derive(Clone, Debug)]
pub struct ActionScore { pub action: ActionPrimitive, pub score: f32 }

#[derive(Clone, Debug)]
pub struct Resident {
    pub id:u64, pub position:Position, pub mind:AgentMind, pub memory:EpisodicMemory,
    pub health:f32, pub current_action:ActionPrimitive, pub top_scores:Vec<ActionScore>, pub awareness:Awareness,
    pub life:LifeHistory, pub knowledge:KnowledgeStore, pub practice:PracticeProfile,
}

#[derive(Clone, Debug)]
pub struct SandboxAnimal { pub id:u64, pub position:Position, pub hunger:f32, pub health:f32, pub archetype:AnimalArchetype }

#[derive(Clone, Debug)]
pub struct SandboxMonster {
    pub id:u64, pub position:Position, pub hunger:f32, pub health:f32, pub archetype:MonsterArchetype, pub skin:Option<PixelSkin>, pub blueprint:Option<MonsterBlueprint>,
    pub motion:MotionState, pub motion_phase:f32,
}

#[derive(Clone, Debug)]
pub struct SandboxFeature {
    pub id:u64, pub kind:FeatureKind, pub position:Position,
    pub danger:f32, pub food:f32, pub material:f32,
    pub quantity:f32, pub capacity:f32, pub regeneration_per_day:f32,
}

#[derive(Clone, Debug)]
pub struct Sandbox {
    pub seed:u64, pub year:f64, pub residents:Vec<Resident>, pub animals:Vec<SandboxAnimal>, pub monsters:Vec<SandboxMonster>,
    pub features:Vec<SandboxFeature>, pub households:Vec<Household>, pub settlements:Vec<SettlementIdentity>, pub projects:Vec<ConstructionProject>, pub structures:Vec<BuiltStructure>, pub events:Vec<WorldEvent>, pub causal_log:CausalLog, pub next_id:u64,
    pub contacts:Vec<Contact>, pub institutions:Vec<Institution>, pub narratives:Vec<Narrative>,
    pub trades:Vec<TradeRecord>, pub trade_count:u64, pub theft_count:u64,
    pub warbands:Vec<Warband>, pub battles:u64, pub monsters_repelled:u64,
    pub ground_truth:GroundTruthRegistry, pub draft_count:u64,
}

fn unit(seed:u64, stream:u64)->f32 {
    let mut x=seed ^ stream.wrapping_mul(0x9E3779B97F4A7C15);
    x^=x>>30; x=x.wrapping_mul(0xBF58476D1CE4E5B9);
    x^=x>>27; x=x.wrapping_mul(0x94D049BB133111EB);
    x^=x>>31;
    (x as f64/u64::MAX as f64) as f32
}
fn signed(seed:u64,stream:u64)->f32 { unit(seed,stream)*2.0-1.0 }
fn dist(a:Position,b:Position)->f32 { ((a.x-b.x).powi(2)+(a.y-b.y).powi(2)).sqrt() }
fn blocked(pos:Position,features:&[SandboxFeature])->bool {
    features.iter().any(|f|{
        let r=match f.kind {FeatureKind::DeepWater=>17.0,FeatureKind::RockFace=>7.0,_=>return false};
        dist(pos,f.position)<r
    })
}
fn try_move(p:&mut Position,dx:f32,dy:f32,amount:f32,features:&[SandboxFeature]) {
    let d=(dx*dx+dy*dy).sqrt(); if d<=0.001{return;}
    let s=amount.min(d)/d; let direct=Position{x:p.x+dx*s,y:p.y+dy*s};
    if !blocked(direct,features){*p=direct;return;}
    let left=Position{x:p.x-dy/d*amount,y:p.y+dx/d*amount};
    let right=Position{x:p.x+dy/d*amount,y:p.y-dx/d*amount};
    if !blocked(left,features){*p=left;} else if !blocked(right,features){*p=right;}
}
fn move_toward(p:&mut Position,target:Position,amount:f32,features:&[SandboxFeature]) {
    try_move(p,target.x-p.x,target.y-p.y,amount,features);
}
fn move_away(p:&mut Position,target:Position,amount:f32,features:&[SandboxFeature]) {
    try_move(p,p.x-target.x,p.y-target.y,amount,features);
}

impl Sandbox {
    pub fn new(seed:u64)->Self {
        let mut residents=Vec::new();
        for i in 0..36u64 {
            let mind=AgentMind{
                traits:Traits{
                    threat_sensitivity:unit(seed,i*20+1), aggression:unit(seed,i*20+2),
                    curiosity:unit(seed,i*20+3), empathy:unit(seed,i*20+4), conformity:unit(seed,i*20+5),
                    persistence:unit(seed,i*20+6), risk_tolerance:unit(seed,i*20+7),
                    novelty_seeking:unit(seed,i*20+8), social_trust:unit(seed,i*20+9),
                    planning_horizon:unit(seed,i*20+10),
                },
                needs:Needs{hunger:0.25+unit(seed,i*20+11)*0.25,safety:0.2,rest:0.15,belonging:0.3,status:0.2,curiosity:0.2+unit(seed,i*20+12)*0.5,care:0.25,resources:0.20},
                ..Default::default()
            };
            residents.push(Resident{
                id:i+1, position:Position{x:-120.0+signed(seed,i*20+13)*55.0,y:signed(seed,i*20+14)*90.0},
                mind,memory:EpisodicMemory{episodes:vec![],capacity:64},health:1.0,
                current_action:ActionPrimitive::Observe,top_scores:vec![],awareness:Awareness::default(),life:LifeHistory{birth_year:-18.0-unit(seed,i*20+15) as f64*28.0,sex:if unit(seed,i*20+22)<0.5{Sex::Female}else{Sex::Male},last_birth_year:None,biological:HeritableTraits{stature:unit(seed,i*20+16),body_mass:unit(seed,i*20+17),cold_tolerance:unit(seed,i*20+18),heat_tolerance:unit(seed,i*20+19),pigmentation:unit(seed,i*20+20),disease_resistance:unit(seed,i*20+21)},kinship:Kinship::default(),social:SocialMemory::default()},knowledge:KnowledgeStore::default(),practice:PracticeProfile::default(),
            });
        }
        let mut features=Vec::new(); let mut id=10_000u64;
        for i in 0..42u64 {
            let capacity=18.0+unit(seed,880+i)*26.0;
            features.push(SandboxFeature{id,kind:FeatureKind::Vegetation,
                position:Position{x:-220.0+unit(seed,500+i)*440.0,y:-160.0+unit(seed,700+i)*320.0},
                danger:0.02,food:0.35+unit(seed,900+i)*0.45,material:0.9,
                quantity:capacity*(0.55+unit(seed,960+i)*0.40),capacity,regeneration_per_day:0.05+unit(seed,980+i)*0.07}); id+=1;
        }
        for i in 0..16u64 {
            let capacity=10.0+unit(seed,1010+i)*28.0;
            features.push(SandboxFeature{id,kind:FeatureKind::LooseMaterial,
                position:Position{x:-220.0+unit(seed,1030+i)*440.0,y:-160.0+unit(seed,1050+i)*320.0},
                danger:0.01,food:0.0,material:0.55+unit(seed,1070+i)*0.40,
                quantity:capacity,capacity,regeneration_per_day:0.0}); id+=1;
        }
        for i in 0..18u64 {
            features.push(SandboxFeature{id,kind:FeatureKind::RockFace,
                position:Position{x:70.0+unit(seed,1100+i)*180.0,y:-180.0+unit(seed,1300+i)*360.0},
                danger:0.08,food:0.0,material:0.8,quantity:120.0,capacity:120.0,regeneration_per_day:0.0}); id+=1;
        }
        for y in -9..=9 {
            features.push(SandboxFeature{id,kind:FeatureKind::DeepWater,
                position:Position{x:0.0,y:y as f32*22.0},danger:0.65,food:0.0,material:0.0,quantity:1.0,capacity:1.0,regeneration_per_day:0.0}); id+=1;
        }
        Self{seed,year:0.0,residents,animals:vec![],monsters:vec![],features,households:vec![],settlements:vec![],projects:vec![],structures:vec![],events:vec![],causal_log:CausalLog{nodes:vec![],capacity:2048,total_written:0},next_id:id,
            contacts:vec![],institutions:vec![],narratives:vec![],trades:vec![],trade_count:0,theft_count:0,
            warbands:vec![],battles:0,monsters_repelled:0,ground_truth:canonical_registry(),draft_count:0}
    }

    pub fn spawn_resident_at(&mut self, position:Position) {
        let id=self.next_id; self.next_id+=1;
        let s=self.seed^id;
        let mind=AgentMind{
            traits:Traits{threat_sensitivity:unit(s,1),aggression:unit(s,2),curiosity:unit(s,3),empathy:unit(s,4),
                conformity:unit(s,5),persistence:unit(s,6),risk_tolerance:unit(s,7),novelty_seeking:unit(s,8),
                social_trust:unit(s,9),planning_horizon:unit(s,10)},
            needs:Needs{hunger:0.25,safety:0.2,rest:0.15,belonging:0.3,status:0.2,curiosity:0.35,care:0.25,resources:0.20},
            ..Default::default()
        };
        self.residents.push(Resident{id,position,mind,memory:EpisodicMemory{episodes:vec![],capacity:64},health:1.0,
            current_action:ActionPrimitive::Observe,top_scores:vec![],awareness:Awareness::default(),life:LifeHistory{birth_year:self.year-18.0-unit(s,11) as f64*22.0,sex:if unit(s,18)<0.5{Sex::Female}else{Sex::Male},last_birth_year:None,biological:HeritableTraits{stature:unit(s,12),body_mass:unit(s,13),cold_tolerance:unit(s,14),heat_tolerance:unit(s,15),pigmentation:unit(s,16),disease_resistance:unit(s,17)},kinship:Kinship::default(),social:SocialMemory::default()},knowledge:KnowledgeStore::default(),practice:PracticeProfile::default(),});
        self.causal_log.push(self.year,CausalNode::WorldEvent{event_id:id,label:"Resident spawned".into()});
    }
    pub fn spawn_animal_at(&mut self, position:Position) { self.spawn_animal_with(position,AnimalArchetype::default()); }
    pub fn spawn_animal_with(&mut self, position:Position, archetype:AnimalArchetype) {
        let id=self.next_id;self.next_id+=1;
        self.animals.push(SandboxAnimal{id,position,hunger:0.35,health:1.0,archetype});
        self.causal_log.push(self.year,CausalNode::WorldEvent{event_id:id,label:"Animal spawned".into()});
    }
    pub fn grow_vegetation_at(&mut self, position:Position, count:u32) {
        for _ in 0..count {
            let id=self.next_id;self.next_id+=1;
            let a=unit(self.seed,id*3)*std::f32::consts::TAU;let r=unit(self.seed,id*3+1)*32.0;
            let capacity=18.0+unit(self.seed,id*7+4)*26.0;
            self.features.push(SandboxFeature{id,kind:FeatureKind::Vegetation,position:Position{x:position.x+a.cos()*r,y:position.y+a.sin()*r},
                danger:0.02,food:0.35+unit(self.seed,id*3+2)*0.45,material:0.9,
                quantity:capacity*0.80,capacity,regeneration_per_day:0.05+unit(self.seed,id*7+5)*0.07});
        }
    }
    pub fn raise_rock_at(&mut self, position:Position, radius:f32) {
        let id=self.next_id;self.next_id+=1;
        self.features.push(SandboxFeature{id,kind:FeatureKind::RockFace,position,
            danger:0.08,food:0.0,material:0.8,quantity:120.0,capacity:120.0,regeneration_per_day:0.0});
        self.causal_log.push(self.year,CausalNode::WorldEvent{event_id:id,label:format!("god raised rock (r={radius:.0})")});
    }
    pub fn dig_water_at(&mut self, position:Position, radius:f32) {
        let id=self.next_id;self.next_id+=1;
        self.features.push(SandboxFeature{id,kind:FeatureKind::DeepWater,position,
            danger:0.65,food:0.0,material:0.0,quantity:1.0,capacity:1.0,regeneration_per_day:0.0});
        self.causal_log.push(self.year,CausalNode::WorldEvent{event_id:id,label:format!("god dug water (r={radius:.0})")});
    }
    pub fn deposit_minerals_at(&mut self, position:Position, count:u32) {
        for _ in 0..count {
            let id=self.next_id;self.next_id+=1;
            let a=unit(self.seed,id*5)*std::f32::consts::TAU;let r=unit(self.seed,id*5+1)*24.0;
            let capacity=12.0+unit(self.seed,id*5+3)*26.0;
            self.features.push(SandboxFeature{id,kind:FeatureKind::LooseMaterial,position:Position{x:position.x+a.cos()*r,y:position.y+a.sin()*r},
                danger:0.01,food:0.0,material:0.65+unit(self.seed,id*5+2)*0.3,
                quantity:capacity,capacity,regeneration_per_day:0.0});
        }
    }

    pub fn inject_event(&mut self, kind:WorldEventKind, position:Position, radius:f32, intensity:f32, duration_days:f32)->u64 {
        let id=self.next_id; self.next_id+=1;
        let event=WorldEvent{id,kind,position,radius:radius.max(1.0),intensity:intensity.max(0.0),start_year:self.year,duration_years:duration_days.max(0.1) as f64/365.0};
        self.events.push(event);
        self.causal_log.push(self.year,CausalNode::WorldEvent{event_id:id,label:format!("{:?} intensity {:.2}",kind,intensity)});
        id
    }

    pub fn spawn_monster(&mut self) {
        let n=self.monsters.len() as u64;
        self.spawn_monster_at(Position{x:130.0+signed(self.seed,3000+n)*80.0,y:signed(self.seed,3200+n)*140.0});
    }
    pub fn spawn_monster_at(&mut self, position:Position) {
        let n=self.monsters.len() as u64; let mut archetype=MonsterArchetype::default(); archetype.aggression=0.55+unit(self.seed,3400+n)*0.4; self.spawn_monster_with(position,archetype,None,None);
    }
    pub fn spawn_monster_with(&mut self, position:Position, archetype:MonsterArchetype, skin:Option<PixelSkin>, blueprint:Option<MonsterBlueprint>) {
        let id=self.next_id; self.next_id+=1;
        self.monsters.push(SandboxMonster{id,position,hunger:0.7,health:1.0,archetype,skin,blueprint,motion:MotionState::Idle,motion_phase:0.0});
        self.causal_log.push(self.year,CausalNode::WorldEvent{event_id:id,label:"Monster spawned".into()});
    }

    pub fn step(&mut self,days:f32) {
        self.year+=days as f64/365.0;
        let snapshot_residents=self.residents.clone();
        let snapshot_projects=self.projects.clone();
        let snapshot_structures=self.structures.clone();
        let snapshot_monsters=self.monsters.clone();
        let snapshot_animals=self.animals.clone();
        let active_events:Vec<_>=self.events.iter().copied().filter(|e|e.active(self.year)).collect();
        let features=self.features.clone();
        let timber_truth=self.timber_truth();
        let mut social_effects:Vec<(u64,u64,ActionPrimitive)>=Vec::new();
        let mut construction_work:Vec<(u64,u64,ActionPrimitive,f32)>=Vec::new();
        let mut maintenance_work:Vec<(u64,u64,ActionPrimitive,f32)>=Vec::new();
        let mut harvests:Vec<(u64,Option<u64>,u64,f32)>=Vec::new();
        for r in &mut self.residents {
            if r.health<=0.0 { continue; }
            // Learned values decay so no single action can ratchet into permanent dominance.
            let decay=(1.0-days*0.004).max(0.0);
            for v in r.mind.learned_action_value.values_mut(){*v*=decay;}
            r.mind.needs.hunger=(r.mind.needs.hunger+days*0.006).clamp(0.0,1.0);
            r.mind.needs.rest=(r.mind.needs.rest+days*0.002).clamp(0.0,1.0);
            r.mind.needs.resources=if let Some(hid)=r.life.kinship.household {
                let stock=self.households.iter().find(|h|h.id==hid).map(|h|h.shared_material).unwrap_or(0.0);
                let target=self.projects.iter().find(|p|p.household_id==hid)
                    .map(|p|(p.material_required-p.material_committed).max(12.0)).unwrap_or(24.0);
                (1.0-stock/target.max(1.0)).clamp(0.0,1.0)
            } else {0.12};
            let mut perceived=Vec::new();
            let shelter_protection=if r.current_action==ActionPrimitive::Hide {
                snapshot_structures.iter().filter(|s|s.integrity>0.2&&dist(r.position,s.position)<10.0)
                    .map(|s|s.integrity*0.65).fold(0.0_f32,f32::max)
            } else {0.0};
            for e in &active_events {
                let influence=e.influence_at(r.position)*(1.0-shelter_protection);
                if influence>0.0 {
                    if let Some(kind)=e.situation() {
                        let confidence=(0.35+influence*0.5).clamp(0.1,1.0);
                        r.awareness.observe(SituationReport{kind,source_id:Some(e.id),perceived_severity:influence,confidence,observed_year:self.year,location:[e.position.x,e.position.y]});
                    }
                    match e.kind {
                        WorldEventKind::Fire=>{r.mind.needs.safety=(r.mind.needs.safety+influence*0.25*days).clamp(0.0,1.0);r.health=(r.health-influence*0.006*days).max(0.0);}
                        WorldEventKind::Flood=>{r.mind.needs.safety=(r.mind.needs.safety+influence*0.18*days).clamp(0.0,1.0);}
                        WorldEventKind::Earthquake=>{r.mind.needs.safety=(r.mind.needs.safety+influence*0.22*days).clamp(0.0,1.0);}
                        WorldEventKind::Storm=>{r.mind.needs.safety=(r.mind.needs.safety+influence*0.12*days).clamp(0.0,1.0);}
                        WorldEventKind::Drought=>{r.mind.needs.hunger=(r.mind.needs.hunger+influence*0.025*days).clamp(0.0,1.0);}
                        _=>{}
                    }
                }
            }
            for f in &features {
                let d=dist(r.position,f.position);
                if d<=85.0 {
                    let availability=(f.quantity/f.capacity.max(0.001)).clamp(0.0,1.0);
                    perceived.push(PerceivedFeature{id:f.id,kind:f.kind,distance_m:d,danger:f.danger,
                        food_hint:f.food*availability,material_hint:f.material*availability,uncertainty:(d/120.0).clamp(0.05,0.8)});
                }
            }
            for a in &snapshot_animals {
                let d=dist(r.position,a.position);
                if a.health>0.0 && d<=75.0 { perceived.push(PerceivedFeature{id:a.id,kind:FeatureKind::Creature,distance_m:d,danger:0.08+a.archetype.fear*0.08,food_hint:0.35,material_hint:0.18,uncertainty:(d/100.0).clamp(0.05,0.7)}); }
            }
            for p in &snapshot_projects {
                let d=dist(r.position,p.position);
                if d<=85.0 {
                    perceived.push(PerceivedFeature{id:p.id,kind:FeatureKind::ConstructionSite,distance_m:d,danger:0.03,
                        food_hint:0.0,material_hint:p.material_committed/p.material_required.max(0.1),uncertainty:0.12});
                }
            }
            for s in &snapshot_structures {
                let d=dist(r.position,s.position);
                if d<=85.0 {
                    let kind=if s.integrity>0.2{FeatureKind::ConstructedObject}else{FeatureKind::LooseMaterial};
                    perceived.push(PerceivedFeature{id:s.id,kind,distance_m:d,danger:(1.0-s.integrity)*0.25,
                        food_hint:0.0,material_hint:(0.3+s.material_invested/60.0).clamp(0.0,1.0),uncertainty:0.05});
                }
            }
            for m in &snapshot_monsters {
                let d=dist(r.position,m.position);
                if m.health>0.0 && d<=110.0 {
                    let confidence=(1.0-d/140.0).clamp(0.1,1.0);
                    let severity=(m.archetype.aggression*m.health*(0.5+m.archetype.body_mass_kg.sqrt()/60.0)).clamp(0.0,1.5);
                    r.awareness.observe(SituationReport{kind:SituationKind::CreatureThreat,source_id:Some(m.id),perceived_severity:severity,confidence,observed_year:self.year,location:[m.position.x,m.position.y]});
                    // A felt predator is a safety need, not just information.
                    r.mind.needs.safety=(r.mind.needs.safety+severity*confidence*0.05*days).clamp(0.0,1.0);
                    perceived.push(PerceivedFeature{id:m.id,kind:FeatureKind::Creature,distance_m:d,
                        danger:severity,food_hint:0.0,material_hint:0.25,uncertainty:(d/140.0).clamp(0.05,0.75)});
                }
            }
            let stage=r.life.stage(self.year);
            let capability=capability_factor(stage);
            let mut affordances=generate(&perceived,LocalCapabilities{
                reach_m:3.0+9.0*capability,
                cutting:0.05*capability,
                digging:0.04*capability,
                carrying:0.2*capability,
                heat_tolerance:0.0,
            });
            let social_targets:Vec<_>=snapshot_residents.iter().filter(|o|o.id!=r.id&&o.health>0.0&&dist(r.position,o.position)<=28.0).map(|o|SocialTarget{
                id:o.id,distance_m:dist(r.position,o.position),stage:o.life.stage(self.year),health:o.health,hunger:o.mind.needs.hunger,safety_need:o.mind.needs.safety,
                relation:r.life.social.relations.get(&o.id).copied().unwrap_or_default(),
            }).collect();
            affordances.extend(generate_social(&social_targets));
            for a in &mut affordances {
                if let Some(tid)=a.target {
                    if a.action==ActionPrimitive::Attack {
                        if let Some(mp)=snapshot_monsters.iter().find(|m|m.id==tid).map(|m|m.position) {
                            // Muster: an armed crowd facing one predator may stand together.
                            // Lone victims still flee; collective defense needs numbers.
                            let muster=snapshot_residents.iter()
                                .filter(|o|o.id!=r.id&&o.health>0.0&&dist(o.position,mp)<40.0).count();
                            a.expected.safety+=0.10*(muster.min(5) as f32);
                            // Armed defenders stand: repertoire weapons raise resolve, lower felt risk.
                            let melee=r.life.kinship.household
                                .and_then(|hid|self.households.iter().find(|h|h.id==hid))
                                .map(|h|timber_truth.map(|t|melee_from_repertoire(&h.designs,&t)).unwrap_or(0.0))
                                .unwrap_or(0.0).clamp(0.0,1.5);
                            a.expected.status+=0.30*melee;
                            a.expected.physical_risk*=1.0-0.45*melee.clamp(0.0,1.0);
                            // Nest defense: a predator at the doorstep threatens kin and stores.
                            let home_close=r.life.kinship.household
                                .and_then(|hid|self.households.iter().find(|h|h.id==hid))
                                .map(|h|dist(h.home,mp)<45.0).unwrap_or(false);
                            if home_close {
                                a.expected.safety+=0.20;
                                a.expected.care+=0.12;
                            }
                        }
                    }
                    if let Some(p)=snapshot_projects.iter().find(|p|p.id==tid) {
                        if r.life.kinship.household==Some(p.household_id) {
                            // Own-household construction satisfies belonging and material need.
                            a.expected.belonging+=0.14;
                            if a.action==ActionPrimitive::Move {a.expected.belonging+=0.15;}
                            a.expected.care+=0.08;
                            a.expected.material+=0.55;
                        }
                    }
                    if let Some(s)=snapshot_structures.iter().find(|s|s.id==tid&&s.integrity>0.2) {
                        let cap=s.capabilities();
                        match a.action {
                            ActionPrimitive::Hide=>{a.expected.safety+=cap.shelter*0.45+cap.defense*0.20;a.expected.rest+=cap.shelter*0.15;}
                            ActionPrimitive::Observe=>{a.expected.knowledge+=cap.observation*0.16+cap.workspace*0.05;}
                            ActionPrimitive::Experiment=>{a.expected.knowledge+=cap.workspace*0.18+cap.observation*0.08;}
                            ActionPrimitive::Bind|ActionPrimitive::Raise=>{a.expected.safety+=cap.defense*0.10;}
                            _=>{}
                        }
                    }
                }
            }
            if let Some(village)=self.settlements.iter().filter(|s|!s.members.is_empty()).find(|s|s.members.contains(&r.id)) {
                for a in &mut affordances { a.local_norm=village.culture.norm(a.action)*0.35; }
            }
            filter_affordances(stage,&mut affordances);
            if affordances.is_empty() { continue; }
            let mut scored:Vec<(usize,f32)>=affordances.iter().enumerate().map(|(i,a)|{
                let mem=r.memory.recalled_value(a.action,self.year);
                let noise=signed(self.seed,r.id.wrapping_mul(100_000)+self.year.to_bits()+i as u64)*0.08;
                (i,r.mind.score(a,noise)+mem*0.25)
            }).collect();
            scored.sort_by(|a,b|b.1.total_cmp(&a.1));
            r.top_scores=scored.iter().take(5).map(|(i,s)|ActionScore{action:affordances[*i].action,score:*s}).collect();
            let chosen=&affordances[scored[0].0];
            let previous_action=r.current_action;
            r.current_action=chosen.action;
            if previous_action!=chosen.action {
                self.causal_log.push(self.year,CausalNode::Decision{resident_id:r.id,action:chosen.action,score:scored[0].1});
            }
            let target=chosen.target.and_then(|id|{
                features.iter().find(|f|f.id==id).map(|f|f.position)
                    .or_else(||snapshot_monsters.iter().find(|m|m.id==id).map(|m|m.position))
                    .or_else(||snapshot_residents.iter().find(|o|o.id==id).map(|o|o.position))
                    .or_else(||snapshot_projects.iter().find(|p|p.id==id).map(|p|p.position))
                    .or_else(||snapshot_structures.iter().find(|s|s.id==id).map(|s|s.position))
            });
            let mobility=mobility_factor(r.life.stage(self.year));
            if let Some(t)=target {
                // Purposeful trips to one's own construction site travel faster than wandering.
                let own_site=chosen.target.and_then(|id|snapshot_projects.iter().find(|p|p.id==id))
                    .map(|p|Some(p.household_id)==r.life.kinship.household).unwrap_or(false);
                let stride=if own_site&&chosen.action==ActionPrimitive::Move{1.6}else{1.0};
                match chosen.action {
                    ActionPrimitive::Avoid=>move_away(&mut r.position,t,days*1.4*mobility,&features),
                    ActionPrimitive::Hide=>move_toward(&mut r.position,t,days*1.15*mobility,&features),
                    ActionPrimitive::Attack=>move_toward(&mut r.position,t,days*1.8*mobility,&features),
                    ActionPrimitive::Move|ActionPrimitive::Assist=>move_toward(&mut r.position,t,days*0.95*stride*mobility,&features),
                    ActionPrimitive::Communicate=>{
                        if chosen.target.map(|id|snapshot_residents.iter().any(|o|o.id==id)).unwrap_or(false) {
                            move_toward(&mut r.position,t,days*0.75*mobility,&features);
                        }
                    }
                    ActionPrimitive::Gather|ActionPrimitive::Carry|ActionPrimitive::Observe|ActionPrimitive::Experiment|
                    ActionPrimitive::Dig|ActionPrimitive::Strike|ActionPrimitive::Cut|ActionPrimitive::Bind|ActionPrimitive::Raise=>move_toward(&mut r.position,t,days*0.7*mobility,&features),
                    _=>{}
                }
            }
            if let Some(pid)=chosen.target {
                if snapshot_projects.iter().any(|p|p.id==pid) && matches!(chosen.action,ActionPrimitive::Bind|ActionPrimitive::Raise|ActionPrimitive::Dig|ActionPrimitive::Carry|ActionPrimitive::Experiment) {
                    construction_work.push((pid,r.id,chosen.action,r.practice.skill(chosen.action)));
                }
                if snapshot_structures.iter().any(|s|s.id==pid) {
                    if chosen.action==ActionPrimitive::Hide {
                        r.mind.needs.safety=(r.mind.needs.safety-0.04*days).max(0.0);
                        r.mind.needs.rest=(r.mind.needs.rest-0.025*days).max(0.0);
                    }
                    if matches!(chosen.action,ActionPrimitive::Bind|ActionPrimitive::Raise) {
                        maintenance_work.push((pid,r.id,chosen.action,r.practice.skill(chosen.action)));
                    }
                }
            }
            if matches!(chosen.action,ActionPrimitive::Assist|ActionPrimitive::Communicate) {
                if let Some(tid)=chosen.target {if snapshot_residents.iter().any(|o|o.id==tid){social_effects.push((r.id,tid,chosen.action));}}
            }
            let value=if chosen.action==ActionPrimitive::Avoid {chosen.expected.safety} else {
                chosen.expected.food+chosen.expected.knowledge+chosen.expected.status+
                chosen.expected.material*r.mind.needs.resources-chosen.expected.physical_risk
            };
            r.mind.learn_action(chosen.action,value,0.04);
            r.practice.practice(chosen.action,value,days,self.year);
            if matches!(chosen.action,ActionPrimitive::Observe|ActionPrimitive::Experiment|ActionPrimitive::Strike|ActionPrimitive::Cut|ActionPrimitive::Dig) {
                r.knowledge.learn(format!("action::{:?}",chosen.action),value,0.18+chosen.expected.knowledge*0.5);
            }
            if matches!(chosen.action,ActionPrimitive::Observe|ActionPrimitive::Experiment) {
                // Satisfied curiosity lowers the drive so observation cannot dominate forever.
                r.mind.needs.curiosity=(r.mind.needs.curiosity-chosen.expected.knowledge*0.35*days.min(1.0)).clamp(0.05,1.0);
                if let Some(tid)=chosen.target {
                    if let Some(s)=snapshot_structures.iter().find(|s|s.id==tid) {
                        let confidence=if chosen.action==ActionPrimitive::Experiment{0.42}else{0.20};
                        r.knowledge.learn(format!("design::{}",s.id),s.integrity,confidence);
                    }
                }
            }
            if chosen.action==ActionPrimitive::Gather {
                if let Some(fid)=chosen.target {
                    if features.iter().any(|f|f.id==fid&&matches!(f.kind,FeatureKind::Vegetation|FeatureKind::LooseMaterial)) {
                        let requested=days.max(0.0)*(0.35+capability*0.65);
                        harvests.push((fid,r.life.kinship.household,r.id,requested));
                    }
                }
            }
            r.memory.remember(Episode{year:self.year,action:chosen.action,target:chosen.target,value,surprise:chosen.uncertainty,
                danger:chosen.expected.physical_risk,social_visibility:0.2});
        }
        for (feature_id,hid,gatherer_id,requested) in harvests {
            let Some(fi)=self.features.iter().position(|f|f.id==feature_id) else{continue;};
            // Sustainable harvest leaves a regrowth base; desperate gatherers strip it bare.
            let desperate=self.residents.iter().find(|r|r.id==gatherer_id).map(|r|r.mind.needs.hunger>0.9).unwrap_or(false);
            let share=if desperate{1.0}else{0.6};
            let taken=requested.min(self.features[fi].quantity.max(0.0)*share);
            if taken<=0.0{continue;}
            self.features[fi].quantity-=taken;
            let food=self.features[fi].food*taken;
            let material=self.features[fi].material*taken;
            if let Some(hid)=hid {
                if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                    let home=h.home;
                    let storage:f32=self.structures.iter().filter(|s|s.household_id==hid&&s.integrity>0.2&&dist(s.position,home)<35.0)
                        .map(|s|s.capabilities().storage).sum();
                    let food_cap=80.0+storage*260.0;
                    let material_cap=35.0+storage*180.0;
                    h.stored_food=(h.stored_food+food).min(food_cap);
                    h.shared_material=(h.shared_material+material).min(material_cap);
                    continue;
                }
            }
            // Household-less foragers eat on the spot instead of starving with full hands.
            if let Some(r)=self.residents.iter_mut().find(|r|r.id==gatherer_id) {
                r.mind.needs.hunger=(r.mind.needs.hunger-food*0.9).max(0.0);
            }
        }

        for (project_id,resident_id,action,skill) in construction_work {
            let Some(pi)=self.projects.iter().position(|p|p.id==project_id) else{continue;};
            if action==ActionPrimitive::Carry {
                let hid=self.projects[pi].household_id;
                if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                    let moved=(0.6+skill*1.4)*days.min(2.0);
                    let amount=moved.min(h.shared_material).min((self.projects[pi].material_required-self.projects[pi].material_committed).max(0.0));
                    h.shared_material-=amount;self.projects[pi].material_committed+=amount;
                }
            }
            let workspace=self.structures.iter().filter(|s|s.integrity>0.2&&dist(s.position,self.projects[pi].position)<28.0)
                .map(|s|s.capabilities().workspace).fold(0.0_f32,f32::max);
            // Coordinated crews with an institution behind them work faster.
            let crew_bonus=self.settlements.iter().find(|s|s.members.contains(&resident_id))
                .map(|s|coordination_bonus(&self.institutions,s.id,action)).unwrap_or(0.0);
            let work=work_value(action,skill,days.min(2.0))*(1.0+workspace*0.35)*(1.0+crew_bonus);
            // Builders haul their own materials as they work; dedicated Carry trips move more.
            if action!=ActionPrimitive::Carry {
                let hid=self.projects[pi].household_id;
                let remaining=(self.projects[pi].material_required-self.projects[pi].material_committed).max(0.0);
                if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                    let haul=(work*0.35).min(h.shared_material).min(remaining);
                    h.shared_material-=haul;self.projects[pi].material_committed+=haul;
                }
            }
            let before=(self.projects[pi].progress/self.projects[pi].required_work.max(0.1)).clamp(0.0,1.0);
            self.projects[pi].progress+=work;
            let after=(self.projects[pi].progress/self.projects[pi].required_work.max(0.1)).clamp(0.0,1.0);
            if (before*4.0).floor()<(after*4.0).floor() {
                self.causal_log.push(self.year,CausalNode::Outcome{resident_id:Some(resident_id),label:format!("construction {} reached {:.0}%",project_id,after*100.0),value:after});
            }
        }

        for (structure_id,resident_id,action,skill) in maintenance_work {
            let Some(si)=self.structures.iter().position(|s|s.id==structure_id) else{continue;};
            if self.structures[si].integrity<=0.2 {continue;}
            let hid=self.structures[si].household_id;
            let work=work_value(action,skill,days.min(2.0));
            let material_need=work*0.22;
            let used=if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                let x=material_need.min(h.shared_material);h.shared_material-=x;x
            } else {0.0};
            if used>0.0 {
                let material_ratio=(used/material_need.max(0.001)).clamp(0.0,1.0);
                let repair=work*0.008*(0.35+0.65*material_ratio);
                self.structures[si].integrity=(self.structures[si].integrity+repair).min(1.0);
                if repair>=0.02 {
                    self.causal_log.push(self.year,CausalNode::Outcome{resident_id:Some(resident_id),label:format!("maintained structure {}",structure_id),value:repair});
                }
            }
        }

        for (actor_id,target_id,action) in social_effects {
            let Some(ai)=self.residents.iter().position(|r|r.id==actor_id) else{continue;};
            let Some(ti)=self.residents.iter().position(|r|r.id==target_id) else{continue;};
            if ai==ti{continue;}
            let (actor,target)=if ai<ti {
                let (l,r)=self.residents.split_at_mut(ti);(&mut l[ai],&mut r[0])
            } else {
                let (l,r)=self.residents.split_at_mut(ai);(&mut r[0],&mut l[ti])
            };
            if actor.health<=0.0||target.health<=0.0||dist(actor.position,target.position)>14.0{continue;}
            match action {
                ActionPrimitive::Assist=>{
                    let care=0.008+actor.mind.traits.empathy*0.020;
                    target.mind.needs.hunger=(target.mind.needs.hunger-care*days).max(0.0);
                    target.mind.needs.safety=(target.mind.needs.safety-care*0.6*days).max(0.0);
                    target.mind.needs.care=(target.mind.needs.care-care*1.2*days).max(0.0);
                    actor.mind.needs.belonging=(actor.mind.needs.belonging-0.08*days.min(1.0)).max(0.05);
                    actor.life.social.observe_help(target.id,(care*10.0).clamp(0.0,1.0));
                    target.life.social.observe_help(actor.id,(care*12.0).clamp(0.0,1.0));
                }
                ActionPrimitive::Communicate=>{
                    let at=actor.mind.traits;let tt=target.mind.traits;
                    spend_time(actor.life.social.relation_mut(target.id),days.min(1.0)*2.0,at,tt);
                    spend_time(target.life.social.relation_mut(actor.id),days.min(1.0)*2.0,tt,at);
                    // Conversation satiates belonging so socializing cannot dominate forever.
                    actor.mind.needs.belonging=(actor.mind.needs.belonging-0.12*days.min(1.0)).max(0.05);
                    target.mind.needs.belonging=(target.mind.needs.belonging-0.12*days.min(1.0)).max(0.05);
                    // Gathering under a shared story eases fear a little.
                    if let Some(sid)=self.settlements.iter().find(|s|s.members.contains(&actor.id)).map(|s|s.id) {
                        let relief=ritual_relief(&self.narratives,sid);
                        if relief>0.0 {
                            actor.mind.needs.safety=(actor.mind.needs.safety-relief*days).max(0.0);
                            target.mind.needs.safety=(target.mind.needs.safety-relief*days).max(0.0);
                        }
                    }
                }
                _=>{}
            }
        }

        // Local teaching: nearby residents can pass imperfect knowledge according to trust.
        let teachers:Vec<_>=self.residents.iter().filter(|r|r.health>0.0&&!r.knowledge.items.is_empty())
            .map(|r|(r.id,r.position,r.knowledge.clone(),r.practice.clone())).collect();
        for r in &mut self.residents {
            for (teacher,pos,knowledge,practice) in &teachers {
                if *teacher==r.id {continue;} let d=dist(r.position,*pos);
                if d<=16.0 && r.mind.traits.social_trust>0.35 {
                    let trust=(0.25+r.mind.traits.social_trust*0.65).clamp(0.0,1.0);
                    let distortion=signed(self.seed,r.id.wrapping_mul(911_003)+*teacher)*0.12;
                    knowledge.transmit_to(&mut r.knowledge,trust,distortion);
                    let learner_stage=r.life.stage(self.year);
                    let teaching_gain=match learner_stage {LifeStage::Child=>1.8,LifeStage::Adolescent=>1.4,_=>0.65};
                    for (action,skill) in practice.dominant(3) {
                        if skill>0.12 {
                            r.practice.practice(action,skill*trust,days*0.10*teaching_gain,self.year);
                            let old=*r.mind.learned_action_value.get(&action).unwrap_or(&0.0);
                            r.mind.learned_action_value.insert(action,old+(skill-old)*0.015*trust*teaching_gain);
                        }
                    }
                }
            }
        }

        // Local hearsay: nearby residents may transmit their strongest creature-threat report.
        let reports:Vec<_>=self.residents.iter().filter_map(|r|r.awareness.reports.get(&SituationKind::CreatureThreat).copied().map(|q|(r.id,r.position,q))).collect();
        for r in &mut self.residents {
            for (source,pos,report) in &reports {
                if *source==r.id {continue;}
                let d=dist(r.position,*pos);
                if d<=38.0 && report.confidence>0.25 {
                    let trust=(0.35+r.mind.traits.social_trust*0.6).clamp(0.0,1.0);
                    let distortion=signed(self.seed,r.id.wrapping_mul(700_001)+*source)*0.22;
                    let before=r.awareness.reports.get(&report.kind).copied();
                    r.awareness.hear(*report,trust,distortion,self.year);
                    let after=r.awareness.reports.get(&report.kind).copied();
                    let meaningful=match (before,after) {
                        (None,Some(_))=>true,
                        (Some(a),Some(b))=>b.confidence>a.confidence+0.08 || (b.perceived_severity-a.perceived_severity).abs()>0.12,
                        _=>false,
                    };
                    if meaningful {
                        self.causal_log.push(self.year,CausalNode::Transmission{from:*source,to:r.id,label:format!("{:?} rumor",report.kind)});
                    }
                }
            }
        }
        for s in &mut self.structures {
            let mut damage=days*0.0000025;
            for e in &active_events {
                let x=e.influence_at(s.position);
                if x<=0.0{continue;}
                damage+=match e.kind {
                    WorldEventKind::Fire=>x*0.010*days,
                    WorldEventKind::Flood=>x*0.004*days,
                    WorldEventKind::Earthquake=>x*0.020*days,
                    WorldEventKind::Storm=>x*0.006*days,
                    _=>0.0,
                };
            }
            s.integrity=(s.integrity-damage).max(0.0);
        }
        for f in &mut self.features {
            if f.regeneration_per_day>0.0 {
                let drought:f32=active_events.iter().filter(|e|e.kind==WorldEventKind::Drought).map(|e|e.influence_at(f.position)).sum();
                let rain:f32=active_events.iter().filter(|e|e.kind==WorldEventKind::Rain).map(|e|e.influence_at(f.position)).sum();
                let fire:f32=active_events.iter().filter(|e|e.kind==WorldEventKind::Fire).map(|e|e.influence_at(f.position)).sum();
                let moisture=(1.0-drought.clamp(0.0,0.95))*(1.0+rain.clamp(0.0,1.5)*0.65);
                let growth=f.regeneration_per_day*days*moisture;
                f.quantity=(f.quantity+growth-fire*0.18*days).clamp(0.0,f.capacity);
            }
        }
        self.events.retain(|e|e.active(self.year));
        for a in &mut self.animals {
            if a.health<=0.0 {continue;} a.hunger=(a.hunger+days*0.002).clamp(0.0,1.0);
            let nearest_monster=self.monsters.iter().filter(|m|m.health>0.0).map(|m|(m.position,dist(a.position,m.position))).min_by(|x,y|x.1.total_cmp(&y.1));
            if let Some((p,d))=nearest_monster {if d<70.0 {move_away(&mut a.position,p,days*(0.35+a.archetype.speed+a.archetype.fear*0.45),&features);continue;}}
            a.position.x+=signed(self.seed,self.year.to_bits()+a.id)*days*0.25;a.position.y+=signed(self.seed,self.year.to_bits()+a.id+3)*days*0.25;
        }
        for m in &mut self.monsters {
            m.motion_phase=(m.motion_phase+days*(0.18+m.archetype.speed*0.22)).fract();
            if m.health<=0.0 {m.motion=MotionState::Death;continue;}
            m.motion=MotionState::Idle;
            m.hunger=(m.hunger+days*0.004).clamp(0.0,1.0);
            if let Some((idx,d))=self.residents.iter().enumerate().filter(|(_,r)|r.health>0.0)
                .map(|(i,r)|(i,dist(m.position,r.position))).min_by(|a,b|a.1.total_cmp(&b.1)) {
                let target=self.residents[idx].position;
                if d<8.0 && m.hunger*0.55+m.archetype.aggression*0.45>0.45 {
                    m.motion=MotionState::Attack;
                    let damage=(0.008+0.020*m.archetype.aggression+0.000012*m.archetype.body_mass_kg)*days;
                    // Hiding inside intact shelter mitigates monster damage.
                    let victim_hide=self.residents[idx].current_action==ActionPrimitive::Hide;
                    let cover=if victim_hide {
                        self.structures.iter()
                            .filter(|s|s.integrity>0.2&&dist(self.residents[idx].position,s.position)<10.0)
                            .map(|s|s.integrity*0.65).fold(0.0_f32,f32::max)
                    } else {0.0};
                    self.residents[idx].health=(self.residents[idx].health-damage*(1.0-cover)).max(0.0);
                    m.hunger=(m.hunger-damage*1.5).max(0.0);
                } else if d<180.0 && m.hunger>0.35 {
                    m.motion=if m.archetype.speed>0.9{MotionState::Run}else{MotionState::Walk};
                    // Committed lunge: close hungry predators sprint faster than fleeing prey.
                    let sprint=if d<25.0&&m.hunger>0.5{2.2}else{1.0};
                    move_toward(&mut m.position,target,days*(0.60+m.archetype.speed*1.8+m.archetype.aggression*0.5)*sprint,&features);
                } else {
                    m.position.x+=signed(self.seed,self.year.to_bits()+m.id)*days*0.4;
                    m.position.y+=signed(self.seed,self.year.to_bits()+m.id+1)*days*0.4;
                }
            }
        }
        self.step_social_generation(days);
        self.step_construction(days);
        self.update_settlement_identities();
        self.step_language_contact();
        self.step_trade_theft(days);
        self.step_institutions_narratives();
        self.step_experimentation(days);
        self.step_warfare(days);
        // Hunting: killing an animal yields meat for the hunter's household, or the hunter.
        let mut hunt_kills:Vec<(u64,Option<u64>,u64,f32)>=Vec::new();
        for r in &mut self.residents {
            if r.current_action==ActionPrimitive::Attack {
                for m in &mut self.monsters {
                    if m.health>0.0 && dist(r.position,m.position)<9.0 {
                        m.health=(m.health-(0.01+0.025*r.mind.traits.aggression)*days).max(0.0);
                    }
                }
                for a in &mut self.animals {
                    if a.health>0.0 && dist(r.position,a.position)<9.0 {
                        a.health=(a.health-(0.02+0.05*r.mind.traits.aggression)*days).max(0.0);
                        if a.health<=0.0 {
                            hunt_kills.push((r.id,r.life.kinship.household,a.id,6.0+a.archetype.body_mass_kg*0.02));
                        }
                    }
                }
            }
        }
        for (rid,hid,aid,meat) in hunt_kills {
            let mut stored=false;
            if let Some(hid)=hid {
                if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                    h.stored_food+=meat;stored=true;
                }
            }
            if !stored {
                if let Some(r)=self.residents.iter_mut().find(|r|r.id==rid) {
                    r.mind.needs.hunger=(r.mind.needs.hunger-meat*0.06).max(0.0);
                }
            }
            self.causal_log.push(self.year,CausalNode::Outcome{resident_id:Some(rid),label:format!("hunted animal {aid}"),value:meat});
        }
    }
    fn step_social_generation(&mut self, days:f32) {
        let year=self.year;
        let initial_len=self.residents.len();

        // Proximity creates familiarity; stable high-affinity relationships can become partnerships.
        let mut new_partnerships:Vec<(usize,usize)>=Vec::new();
        for i in 0..initial_len {
            for j in (i+1)..initial_len {
                let (left,right)=self.residents.split_at_mut(j);
                let a=&mut left[i]; let b=&mut right[0];
                if a.health<=0.0||b.health<=0.0||dist(a.position,b.position)>12.0 {continue;}
                let at=a.mind.traits; let bt=b.mind.traits;
                {
                    let ra=a.life.social.relation_mut(b.id);
                    spend_time(ra,days.min(1.0)*3.0,at,bt);
                }
                {
                    let rb=b.life.social.relation_mut(a.id);
                    spend_time(rb,days.min(1.0)*3.0,bt,at);
                }
                let adult_a=a.life.stage(year)==LifeStage::Adult;
                let adult_b=b.life.stage(year)==LifeStage::Adult;
                if adult_a&&adult_b&&!a.life.kinship.partners.contains(&b.id) {
                    let rel=a.life.social.relations.get(&b.id).copied().unwrap_or_default();
                    let affinity=partnership_affinity(rel,at,bt);
                    let chance=(days/365.0*0.45*((affinity-0.72)/0.28).clamp(0.0,1.0)).clamp(0.0,0.05);
                    let roll=unit(self.seed,a.id.wrapping_mul(1_000_003)^b.id^year.to_bits());
                    if affinity>0.72&&roll<chance {new_partnerships.push((i,j));}
                }
            }
        }
        for (i,j) in new_partnerships {
            let (left,right)=self.residents.split_at_mut(j);
            let a=&mut left[i]; let b=&mut right[0];
            if !a.life.kinship.partners.contains(&b.id){a.life.kinship.partners.push(b.id);}
            if !b.life.kinship.partners.contains(&a.id){b.life.kinship.partners.push(a.id);}
            match (a.life.kinship.household,b.life.kinship.household) {
                (None,None)=>{
                    let hid=self.next_id;self.next_id+=1;
                    self.households.push(Household{id:hid,members:vec![a.id,b.id],home:Position{x:(a.position.x+b.position.x)*0.5,y:(a.position.y+b.position.y)*0.5},stored_food:80.0,shared_material:0.0,cohesion:0.55,migration_goal:None,construction_drive:0.0,designs:vec![]});
                    a.life.kinship.household=Some(hid);b.life.kinship.household=Some(hid);
                }
                (Some(h),None)=>{b.life.kinship.household=Some(h);if let Some(hh)=self.households.iter_mut().find(|x|x.id==h){if !hh.members.contains(&b.id){hh.members.push(b.id);}}},
                (None,Some(h))=>{a.life.kinship.household=Some(h);if let Some(hh)=self.households.iter_mut().find(|x|x.id==h){if !hh.members.contains(&a.id){hh.members.push(a.id);}}},
                _=>{}
            }
            self.causal_log.push(year,CausalNode::Outcome{resident_id:Some(a.id),label:format!("partnership formed with {}",b.id),value:1.0});
        }

        // Children preferentially learn from parents/guardians when nearby.
        let parent_teaching:Vec<_>=self.residents.iter().filter(|r|r.health>0.0)
            .map(|r|(r.id,r.position,r.knowledge.clone(),r.life.social.relations.clone())).collect();
        for child in &mut self.residents {
            if child.health<=0.0 || !matches!(child.life.stage(year),LifeStage::Child|LifeStage::Adolescent){continue;}
            for parent_id in child.life.kinship.parents.clone() {
                if let Some((_,pos,knowledge,_))=parent_teaching.iter().find(|(id,_,_,_)|*id==parent_id) {
                    if dist(child.position,*pos)<=18.0 {
                        let distortion=signed(self.seed,child.id.wrapping_mul(77_777)^parent_id)*0.05;
                        knowledge.transmit_to(&mut child.knowledge,0.92,distortion);
                    }
                }
            }
        }

        // Build conception plans first, then mutate/push to avoid borrow conflicts.
        let mut births:Vec<(usize,usize)>=Vec::new();
        for i in 0..initial_len {
            let mother=&self.residents[i];
            if mother.health<=0.0||mother.life.sex!=Sex::Female||mother.life.stage(year)!=LifeStage::Adult {continue;}
            if mother.life.last_birth_year.map(|y|year-y<1.5).unwrap_or(false){continue;}
            let mut best:Option<(usize,f32)>=None;
            for partner_id in &mother.life.kinship.partners {
                let Some(j)=self.residents.iter().position(|r|r.id==*partner_id) else{continue;};
                let partner=&self.residents[j];
                if partner.health<=0.0||partner.life.sex!=Sex::Male||partner.life.stage(year)!=LifeStage::Adult||dist(mother.position,partner.position)>24.0{continue;}
                let rel=mother.life.social.relations.get(partner_id).copied().unwrap_or_default();
                let score=rel.trust*0.45+rel.affection*0.55;
                if best.map(|(_,s)|score>s).unwrap_or(true){best=Some((j,score));}
            }
            let Some((j,_))=best else{continue;};
            let rel=mother.life.social.relations.get(&self.residents[j].id).copied().unwrap_or_default();
            let hh_pressure=mother.life.kinship.household.and_then(|h|self.households.iter().find(|x|x.id==h)).map(|h|h.pressure()).unwrap_or(0.35);
            let prop=conception_propensity(ReproductionContext{stage:mother.life.stage(year),sex:mother.life.sex,health:mother.health,
                hunger:mother.mind.needs.hunger,safety_need:mother.mind.needs.safety,care_trait:mother.mind.traits.empathy,
                household_pressure:hh_pressure,partner_relation:rel});
            let chance=(prop*days/365.0).clamp(0.0,0.02);
            let roll=unit(self.seed,mother.id.wrapping_mul(31_337)^self.residents[j].id^year.to_bits());
            if roll<chance {births.push((i,j));}
        }

        for (mi,fi) in births {
            let mother=self.residents[mi].clone(); let father=self.residents[fi].clone();
            let id=self.next_id;self.next_id+=1; let s=self.seed^id^year.to_bits();
            let variation6=[signed(s,1),signed(s,2),signed(s,3),signed(s,4),signed(s,5),signed(s,6)];
            let variation10=[signed(s,11),signed(s,12),signed(s,13),signed(s,14),signed(s,15),signed(s,16),signed(s,17),signed(s,18),signed(s,19),signed(s,20)];
            let biological=inherit(mother.life.biological,father.life.biological,variation6);
            let traits=inherit_personality(mother.mind.traits,father.mind.traits,variation10);
            let household=mother.life.kinship.household.or(father.life.kinship.household);
            let position=Position{x:(mother.position.x+father.position.x)*0.5+signed(s,21)*2.0,y:(mother.position.y+father.position.y)*0.5+signed(s,22)*2.0};
            // Newborns do not inherit cultural knowledge. It is acquired later through observation and teaching.
            let knowledge=KnowledgeStore::default();
            self.residents.push(Resident{id,position,mind:AgentMind{traits,needs:Needs{hunger:0.15,safety:0.35,rest:0.35,belonging:0.65,status:0.0,curiosity:0.35,care:0.0,resources:0.10},..Default::default()},
                memory:EpisodicMemory{episodes:vec![],capacity:64},health:1.0,current_action:ActionPrimitive::Observe,top_scores:vec![],awareness:Awareness::default(),
                life:LifeHistory{birth_year:year,sex:if unit(s,25)<0.5{Sex::Female}else{Sex::Male},last_birth_year:None,biological,
                    kinship:Kinship{parents:vec![mother.id,father.id],children:vec![],partners:vec![],household},social:SocialMemory::default()},knowledge,practice:PracticeProfile::default()});
            if let Some(m)=self.residents.get_mut(mi){m.life.last_birth_year=Some(year);m.life.kinship.children.push(id);}
            if let Some(f)=self.residents.get_mut(fi){f.life.kinship.children.push(id);}
            if let Some(h)=household.and_then(|h|self.households.iter_mut().find(|x|x.id==h)){h.members.push(id);}
            self.causal_log.push(year,CausalNode::Outcome{resident_id:Some(id),label:format!("born to {} and {}",mother.id,father.id),value:1.0});
        }

        // Aging/health mortality remains individual and stochastic.
        for r in &mut self.residents {
            if r.health<=0.0{continue;}
            let annual=annual_mortality_risk(&r.life,year,r.health);
            let chance=(annual*days/365.0).clamp(0.0,0.5);
            let roll=unit(self.seed,r.id.wrapping_mul(8_388_593)^year.to_bits());
            if roll<chance {
                r.health=0.0;
                self.causal_log.push(year,CausalNode::Outcome{resident_id:Some(r.id),label:format!("died at age {:.1}",r.life.age(year)),value:-1.0});
            }
        }

        // Remove dead residents from active household resource pressure while preserving their kinship history.
        let living_ids:std::collections::BTreeSet<u64>=self.residents.iter().filter(|r|r.health>0.0).map(|r|r.id).collect();
        for h in &mut self.households { h.members.retain(|id|living_ids.contains(id)); }

        // Household food use and optional migration. Pressure creates opportunity, not a forced response.
        for h in &mut self.households {
            let member_ids=h.members.clone();
            for id in member_ids {
                let Some(r)=self.residents.iter_mut().find(|r|r.id==id&&r.health>0.0) else{continue;};
                let demand=(0.018+r.mind.needs.hunger*0.055)*days;
                let eaten=demand.min(h.stored_food);
                h.stored_food-=eaten;
                r.mind.needs.hunger=(r.mind.needs.hunger-eaten*0.9).max(0.0);
                // Prolonged near-max hunger damages health; adequate feeding slowly heals.
                if r.mind.needs.hunger>0.90 {
                    r.health=(r.health-(r.mind.needs.hunger-0.90)*0.008*days).max(0.0);
                } else if r.mind.needs.hunger<0.35 {
                    r.health=(r.health+0.003*days).min(1.0);
                }
            }
        }

        let mut proposed_goals:Vec<(u64,Position)>=Vec::new();
        for h in &self.households {
            if h.migration_goal.is_some(){continue;}
            let members:Vec<&Resident>=self.residents.iter().filter(|r|r.health>0.0&&r.life.kinship.household==Some(h.id)).collect();
            if members.is_empty(){continue;}
            let pressure=h.pressure();
            let disposition=members.iter().map(|r|r.mind.traits.risk_tolerance*0.35+r.mind.traits.novelty_seeking*0.40+r.mind.traits.planning_horizon*0.25).sum::<f32>()/members.len() as f32;
            let trigger=((pressure-0.40).max(0.0)*disposition*days/365.0*1.8).clamp(0.0,0.03);
            let roll=unit(self.seed,h.id.wrapping_mul(4_294_967)^year.to_bits());
            if roll>=trigger{continue;}
            let mut best:Option<(Position,f32)>=None;
            for f in self.features.iter().filter(|f|matches!(f.kind,FeatureKind::Vegetation|FeatureKind::LooseMaterial)) {
                let d=dist(h.home,f.position);
                if d<55.0||d>320.0{continue;}
                let crowd=self.residents.iter().filter(|r|r.health>0.0&&dist(r.position,f.position)<40.0).count() as f32;
                let score=f.food*1.4+f.material*0.55-f.danger*1.2-crowd*0.035-d*0.0008;
                if best.map(|(_,s)|score>s).unwrap_or(true){best=Some((f.position,score));}
            }
            if let Some((goal,_))=best{proposed_goals.push((h.id,goal));}
        }
        for (hid,goal) in proposed_goals {
            if let Some(h)=self.households.iter_mut().find(|h|h.id==hid){
                h.migration_goal=Some(goal);
                self.causal_log.push(year,CausalNode::Outcome{resident_id:None,label:format!("household {} began migration",hid),value:0.2});
            }
        }

        let goals:Vec<(u64,Position)>=self.households.iter().filter_map(|h|h.migration_goal.map(|g|(h.id,g))).collect();
        let waters:Vec<(f32,f32,f32)>=self.features.iter()
            .filter(|f|f.kind==FeatureKind::DeepWater).map(|f|(f.position.x,f.position.y,17.0)).collect();
        for (hid,goal) in goals {
            // Overseas colonization: water crossings need real craft, never assumed.
            let craft=self.household_craft(hid);
            for r in self.residents.iter_mut().filter(|r|r.health>0.0&&r.life.kinship.household==Some(hid)) {
                let rate=days*(0.20+r.mind.traits.persistence*0.22);
                let sails=crosses_water((r.position.x,r.position.y),(goal.x,goal.y),&waters)
                    .map(|sev|can_embark(craft,sev)).unwrap_or(false);
                if sails {
                    move_toward(&mut r.position,goal,rate,&[]);
                } else {
                    move_toward(&mut r.position,goal,rate,&self.features);
                }
            }
            let arrived=self.residents.iter().filter(|r|r.health>0.0&&r.life.kinship.household==Some(hid)).all(|r|dist(r.position,goal)<12.0);
            if arrived {
                if let Some(h)=self.households.iter_mut().find(|h|h.id==hid){
                    h.home=goal;h.migration_goal=None;h.cohesion=(h.cohesion+0.05).clamp(0.0,1.0);
                    self.causal_log.push(year,CausalNode::Outcome{resident_id:None,label:format!("household {} settled new site",hid),value:0.6});
                }
            }
        }
    }

    fn update_settlement_identities(&mut self) {
        use std::collections::{BTreeMap,BTreeSet};
        let alive:Vec<&Resident>=self.residents.iter().filter(|r|r.health>0.0).collect();
        let positions:Vec<Position>=alive.iter().map(|r|r.position).collect();
        let resident_ids:Vec<u64>=alive.iter().map(|r|r.id).collect();
        let clusters=detect_settlements(&positions,75.0,4);
        let previous_membership:BTreeMap<u64,u64>=self.settlements.iter().flat_map(|s|s.members.iter().map(move |id|(*id,s.id))).collect();
        let mut seen:BTreeSet<u64>=BTreeSet::new();

        for cluster in clusters {
            let ids=cluster_member_ids(&cluster,&resident_ids);
            let idx=self.settlements.iter().enumerate().filter(|(_,s)|!seen.contains(&s.id)).filter_map(|(i,s)|{
                let d=dist(cluster.center,s.center);(d<=120.0).then_some((i,d))
            }).min_by(|a,b|a.1.total_cmp(&b.1)).map(|x|x.0);
            let si=if let Some(i)=idx {i} else {
                let mut ancestry:BTreeMap<u64,usize>=BTreeMap::new();
                for rid in &ids {if let Some(parent)=previous_membership.get(rid){*ancestry.entry(*parent).or_default()+=1;}}
                let parent_id=ancestry.into_iter().max_by_key(|(_,n)|*n).map(|(id,_)|id);
                let id=self.next_id; self.next_id+=1;
                self.settlements.push(SettlementIdentity{
                    id,center:cluster.center,founded_year:self.year,last_seen_year:self.year,parent_id,members:vec![],
                    culture:CulturalField::default(),lexicon:Lexicon::seed_from(id),shared_food:0.0,shared_material:0.0,knowledge_items:0,specialization:BTreeMap::new(),
                });
                let label=parent_id.map(|p|format!("settlement {} split from {}",id,p)).unwrap_or_else(||format!("settlement {} emerged",id));
                self.causal_log.push(self.year,CausalNode::Outcome{resident_id:None,label,value:0.7});
                self.settlements.len()-1
            };
            let identity=&mut self.settlements[si];
            seen.insert(identity.id);
            identity.center=cluster.center; identity.last_seen_year=self.year; identity.members=ids.clone();

            let local:Vec<&Resident>=ids.iter().filter_map(|id|self.residents.iter().find(|r|r.id==*id&&r.health>0.0)).collect();
            let mut knowledge:BTreeSet<String>=BTreeSet::new();
            let mut skills:BTreeMap<ActionPrimitive,(f32,u32)>=BTreeMap::new();
            for r in &local {
                for key in r.knowledge.items.keys(){knowledge.insert(key.clone());}
                let success=*r.mind.learned_action_value.get(&r.current_action).unwrap_or(&0.0);
                let prestige=r.life.social.relations.values().map(|x|x.prestige.max(0.0)).fold(0.0,f32::max);
                identity.culture.observe(r.current_action,success,prestige,0.35);
                for (&a,s) in &r.practice.actions {
                    let e=skills.entry(a).or_insert((0.0,0));e.0+=s.skill;e.1+=1;
                }
            }
            identity.knowledge_items=knowledge.len();
            identity.specialization.clear();
            for (a,(sum,n)) in skills {if n>0{identity.specialization.insert(a,sum/n as f32);}}
            identity.shared_food=self.households.iter().filter(|h|dist(h.home,identity.center)<=55.0).map(|h|h.stored_food).sum();
            identity.shared_material=self.households.iter().filter(|h|dist(h.home,identity.center)<=55.0).map(|h|h.shared_material).sum();
        }

        for s in &mut self.settlements {
            if !seen.contains(&s.id) && self.year-s.last_seen_year>2.0 {s.members.clear();}
        }
    }

    fn step_construction(&mut self,days:f32) {
        // Proposals arise from local need, practiced construction actions and stored material.
        let mut proposals:Vec<(u64,Position,f32)>=Vec::new();
        let mut drive_updates:Vec<(u64,Position,f32,f32)>=Vec::new();
        for h in &self.households {
            if h.members.is_empty(){continue;}
            if self.projects.iter().any(|p|p.household_id==h.id){continue;}
            if self.structures.iter().any(|s|s.household_id==h.id&&s.integrity>0.2&&dist(s.position,h.home)<24.0){continue;}
            let members:Vec<&Resident>=self.residents.iter().filter(|r|r.health>0.0&&r.life.kinship.household==Some(h.id)).collect();
            if members.is_empty(){continue;}
            let safety=members.iter().map(|r|r.mind.needs.safety).sum::<f32>()/members.len() as f32;
            let rest=members.iter().map(|r|r.mind.needs.rest).sum::<f32>()/members.len() as f32;
            let skill=members.iter().map(|r|{
                r.practice.skill(ActionPrimitive::Bind).max(r.practice.skill(ActionPrimitive::Raise)).max(r.practice.skill(ActionPrimitive::Dig))
            }).sum::<f32>()/members.len() as f32;
            let strength=proposal_strength(safety,rest,skill,h.shared_material);
            let temperament=0.75+unit(self.seed,h.id.wrapping_mul(5_000_011))*0.50;
            let increment=strength*days.max(0.0)/45.0*temperament;
            drive_updates.push((h.id,h.home,skill,increment));
        }
        for (hid,home,skill,increment) in drive_updates {
            if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                h.construction_drive=(h.construction_drive+increment).min(2.0);
                if h.construction_drive>=1.0 {
                    h.construction_drive=0.0;
                    proposals.push((hid,home,skill));
                }
            }
        }
        for (hid,pos,skill) in proposals {
            let id=self.next_id;self.next_id+=1;
            let available=self.households.iter().find(|h|h.id==hid).map(|h|h.shared_material).unwrap_or(0.0);
            let members:Vec<&Resident>=self.residents.iter().filter(|r|r.health>0.0&&r.life.kinship.household==Some(hid)).collect();
            let mut parent_score:Option<(&BuiltStructure,f32)>=None;
            for s in &self.structures {
                let key=format!("design::{}",s.id);
                let learned=members.iter().filter_map(|r|r.knowledge.items.get(&key)).map(|k|k.confidence).fold(0.0_f32,f32::max);
                let own=if s.household_id==hid{0.35}else{0.0};
                let score=learned+own+(s.integrity*0.15);
                if score>0.20&&parent_score.map(|(_,best)|score>best).unwrap_or(true){parent_score=Some((s,score));}
            }
            let design=if let Some((parent,_))=parent_score {
                let variation=signed(self.seed,id.wrapping_mul(13_337)^self.year.to_bits());
                evolve_shelter_design(&parent.design,id,skill,available,variation)
            } else {
                seed_shelter_design(id,None,0,skill,available)
            };
            // Real threat pressure diverts a proposal into a defensive wall.
            let threatened=members.iter().filter(|r|r.awareness.confidence(crate::awareness::SituationKind::CreatureThreat)>0.5).count();
            let design=if threatened.max(1) as f32/members.len().max(1) as f32>0.3&&threatened>=2 {
                seed_wall_design(id,skill,available)
            } else {design};
            let required_material=(16.0+design.length_m*design.width_m*1.6).clamp(14.0,45.0);
            let required_work=(10.0+design.length_m*design.width_m*1.2).clamp(10.0,32.0);
            let initial=if let Some(h)=self.households.iter_mut().find(|h|h.id==hid){
                let x=h.shared_material.min(required_material*0.10);h.shared_material-=x;x
            }else{0.0};
            self.projects.push(ConstructionProject{id,household_id:hid,position:pos,design,progress:0.0,required_work,
                material_committed:initial,material_required:required_material,started_year:self.year});
            self.causal_log.push(self.year,CausalNode::Outcome{resident_id:None,label:format!("construction {} proposed by household {}",id,hid),value:0.3});
        }

        let mut completed=Vec::new();
        for (i,p) in self.projects.iter().enumerate() {
            if p.progress>=p.required_work && p.material_committed>=p.material_required*0.80 {completed.push(i);}
        }
        for i in completed.into_iter().rev() {
            let p=self.projects.remove(i);
            let local:Vec<&Resident>=self.residents.iter().filter(|r|r.health>0.0&&r.life.kinship.household==Some(p.household_id)).collect();
            let skill=if local.is_empty(){0.0}else{local.iter().map(|r|r.practice.skill(ActionPrimitive::Raise).max(r.practice.skill(ActionPrimitive::Bind))).sum::<f32>()/local.len() as f32};
            let integrity=integrity_from(&p.design,skill,p.material_committed/p.material_required.max(0.1));
            self.structures.push(BuiltStructure{id:p.id,household_id:p.household_id,position:p.position,design:p.design,integrity,completed_year:self.year,material_invested:p.material_committed});
            self.causal_log.push(self.year,CausalNode::Outcome{resident_id:None,label:format!("construction {} completed with integrity {:.2}",p.id,integrity),value:integrity});
        }
    }

    fn step_language_contact(&mut self) {
        let seed=self.seed; let year=self.year;
        // Independent drift first: isolation diverges vocabularies.
        for (k,s) in self.settlements.iter_mut().enumerate() {
            if s.members.is_empty(){continue;}
            s.lexicon.drift(seed,(k as u64).wrapping_mul(2_654_435_761).wrapping_add(year.to_bits()),0.0001);
        }
        // Contact blends vocabularies and opens the diplomacy ledger.
        let n=self.settlements.len();
        for i in 0..n {
            for j in (i+1)..n {
                if self.settlements[i].members.is_empty()||self.settlements[j].members.is_empty(){continue;}
                let d=dist(self.settlements[i].center,self.settlements[j].center);
                if d>150.0{continue;}
                let (a,b)=(self.settlements[i].id,self.settlements[j].id);
                let before=self.contacts.len();
                contact_between(&mut self.contacts,a,b,year);
                if self.contacts.len()>before {
                    self.causal_log.push(year,CausalNode::Outcome{resident_id:None,label:format!("settlements {a} and {b} made contact"),value:0.4});
                }
                let (left,right)=self.settlements.split_at_mut(j);
                let (si,sj)=(&mut left[i],&mut right[0]);
                si.lexicon.blend_toward(&sj.lexicon,0.02,seed,year.to_bits()^0xA5A5);
                sj.lexicon.blend_toward(&si.lexicon,0.02,seed,year.to_bits()^0x5A5A);
            }
        }
    }

    fn households_near(&self,center:Position,radius:f32)->Vec<u64> {
        self.households.iter().filter(|h|dist(h.home,center)<=radius).map(|h|h.id).collect()
    }

    fn step_trade_theft(&mut self, days:f32) {
        let year=self.year; let seed=self.seed;
        // Trade: surplus flows toward deficit between nearby settlements, never by script.
        let snaps:Vec<(u64,Position,f32,f32,usize)>=self.settlements.iter()
            .filter(|s|!s.members.is_empty())
            .map(|s|(s.id,s.center,s.shared_food,s.shared_material,s.members.len())).collect();
        for i in 0..snaps.len() {
            for j in (i+1)..snaps.len() {
                let (a,b)=(snaps[i],snaps[j]);
                if dist(a.1,b.1)>150.0{continue;}
                // Markets meet occasionally, not every step.
                if stream_unit(seed,a.0.wrapping_mul(6_469_693).wrapping_add(b.0).wrapping_add(year.to_bits()))>=0.15{continue;}
                // Caravans haul what ordinary bodies can carry.
                let caravan=carry_capacity(0.5,0.3);
                for (stock_a,stock_b,reserve,carry,food) in [(a.2,b.2,20.0,caravan.food,true),(a.3,b.3,15.0,caravan.material,false)] {
                    for (giver,taker) in [(i,j),(j,i)] {
                        let (gs,ts)=if giver==i{(stock_a,stock_b)}else{(stock_b,stock_a)};
                        let gm=if giver==i{snaps[i].4}else{snaps[j].4};
                        let tm=if giver==i{snaps[j].4}else{snaps[i].4};
                        let amt=exchange_amount(surplus(gs,gm,reserve),-surplus(ts,tm,reserve),carry);
                        if amt<=0.05{continue;}
                        let (gid,tid)=(snaps[giver].0,snaps[taker].0);
                        let gcenter=snaps[giver].1; let tcenter=snaps[taker].1;
                        let givers=self.households_near(gcenter,55.0);
                        let takers=self.households_near(tcenter,55.0);
                        if givers.is_empty()||takers.is_empty(){continue;}
                        let mut left=amt;
                        for hid in givers {
                            if left<=0.0{break;}
                            if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                                let x=if food{h.stored_food}else{h.shared_material};
                                let take=x.min(left); 
                                if food{h.stored_food-=take;}else{h.shared_material-=take;}
                                left-=take;
                            }
                        }
                        let moved=amt-left;
                        if moved<=0.0{continue;}
                        let per=move_per_household(moved,takers.len());
                        for hid in takers {
                            if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                                if food{h.stored_food+=per;}else{h.shared_material+=per;}
                            }
                        }
                        contact_between(&mut self.contacts,gid,tid,year).note_exchange(moved);
                        self.trade_count+=1;
                        self.trades.push(TradeRecord{year,from_settlement:gid,to_settlement:tid,
                            food:if food{moved}else{0.0},material:if food{0.0}else{moved}});
                        if self.trades.len()>256{self.trades.drain(0..64);}
                        if moved>3.0 {
                            self.causal_log.push(year,CausalNode::Outcome{resident_id:None,
                                label:format!("settlement {gid} traded {moved:.1} {} to {tid}",if food{"food"}else{"material"}),value:0.3});
                        }
                    }
                }
            }
        }
        // Theft: the very hungry may take from a nearby foreign store. Witnesses make it diplomacy.
        let thieves:Vec<(u64,Position,Option<u64>)>=self.residents.iter()
            .filter(|r|r.health>0.0&&r.mind.needs.hunger>0.85)
            .map(|r|(r.id,r.position,r.life.kinship.household)).collect();
        for (rid,pos,own) in thieves {
            let mut best:Option<(u64,f32)>=None;
            for h in &self.households {
                if Some(h.id)==own||h.stored_food<10.0{continue;}
                let d=dist(pos,h.home);
                if d<25.0&&best.map(|(_,bd)|d<bd).unwrap_or(true){best=Some((h.id,d));}
            }
            let Some((hid,_))=best else{continue;};
            let roll=stream_unit(seed,rid.wrapping_mul(3_824_908_129).wrapping_add(year.to_bits()));
            if roll>=0.03*days.min(2.0){continue;}
            // Even theft respects hauling limits.
            let cap=self.residents.iter().find(|r|r.id==rid)
                .map(|r|carry_capacity(r.life.biological.stature,r.practice.skill(ActionPrimitive::Carry)).food)
                .unwrap_or(3.0);
            if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                let take=2.0f32.min(h.stored_food).min(cap); h.stored_food-=take;
                if let Some(r)=self.residents.iter_mut().find(|r|r.id==rid) {
                    r.mind.needs.hunger=(r.mind.needs.hunger-take*0.45).max(0.0);
                }
            }
            self.theft_count+=1;
            let witnesses=self.residents.iter()
                .filter(|r|r.health>0.0&&r.id!=rid&&r.life.kinship.household==Some(hid)&&dist(r.position,pos)<25.0)
                .count();
            if witnesses>0 {
                let thief_set=self.settlements.iter().find(|s|s.members.contains(&rid)).map(|s|s.id);
                let victim_set=self.settlements.iter().filter(|s|dist(s.center,self.households.iter().find(|h|h.id==hid).map(|h|h.home).unwrap_or(pos))<60.0).map(|s|s.id).next();
                if let (Some(a),Some(b))=(thief_set,victim_set) {
                    if a!=b {
                        contact_between(&mut self.contacts,a,b,year).note_conflict(0.3);
                        self.causal_log.push(year,CausalNode::Outcome{resident_id:Some(rid),label:format!("caught stealing from settlement {b}"),value:-0.5});
                    }
                }
            } else {
                self.causal_log.push(year,CausalNode::Outcome{resident_id:Some(rid),label:"food went missing".into(),value:-0.2});
            }
        }
    }

    fn step_institutions_narratives(&mut self) {
        use std::collections::BTreeMap;
        let year=self.year;
        // Institutions crystallize from coordinated action within each settlement.
        let mut counts:BTreeMap<u64,BTreeMap<ActionPrimitive,u32>>=BTreeMap::new();
        let mut sizes:BTreeMap<u64,usize>=BTreeMap::new();
        for s in &self.settlements {
            if s.members.is_empty(){continue;}
            sizes.insert(s.id,s.members.len());
            let entry=counts.entry(s.id).or_default();
            for id in &s.members {
                if let Some(r)=self.residents.iter().find(|r|r.id==*id&&r.health>0.0) {
                    *entry.entry(r.current_action).or_default()+=1;
                }
            }
        }
        for (sid,map) in &counts {
            let members=sizes.get(sid).copied().unwrap_or(1);
            update_institutions(*sid,map,members,&mut self.institutions,year);
        }
        // Narratives grow where disasters strike without confident understanding.
        let disasters:Vec<(WorldEventKind,Position,f32)>=self.events.iter().copied()
            .filter(|e|e.active(year)&&matches!(e.kind,WorldEventKind::Fire|WorldEventKind::Flood|WorldEventKind::Earthquake|WorldEventKind::Storm))
            .map(|e|(e.kind,e.position,e.radius)).collect();
        if disasters.is_empty(){return;}
        let mut seen:Vec<(u64,WorldEventKind)>=Vec::new();
        let members:Vec<(u64,Position,u64,Awareness)>=self.residents.iter().filter(|r|r.health>0.0)
            .filter_map(|r|{
                let sid=self.settlements.iter().find(|s|s.members.contains(&r.id)).map(|s|s.id)?;
                Some((r.id,r.position,sid,r.awareness.clone()))
            }).collect();
        for (kind,pos,radius) in disasters {
            for sid in self.settlements.iter().filter(|s|!s.members.is_empty()).map(|s|s.id) {
                if seen.contains(&(sid,kind)){continue;}
                seen.push((sid,kind));
                let mut affected=0usize; let mut confident=0usize;
                for (_,p,msid,aw) in &members {
                    if *msid!=sid{continue;}
                    let dx=p.x-pos.x; let dy=p.y-pos.y;
                    if (dx*dx+dy*dy).sqrt()>=radius{continue;}
                    affected+=1;
                    let want=match kind {
                        WorldEventKind::Fire=>SituationKind::Fire,
                        WorldEventKind::Flood=>SituationKind::Flood,
                        _=>SituationKind::UnknownPhenomenon,
                    };
                    if aw.confidence(want)>=0.5{confident+=1;}
                }
                let mass=misattribution(affected,confident);
                let label=match kind {
                    WorldEventKind::Fire=>"fire spirits hunger",
                    WorldEventKind::Flood=>"river spirits anger",
                    WorldEventKind::Earthquake=>"earth spirits woke",
                    _=>"storm spirits quarrel",
                }.to_string();
                let nmembers=self.settlements.iter().find(|s|s.id==sid).map(|s|s.members.len() as u32).unwrap_or(0);
                reinforce(&mut self.narratives,sid,label,mass,nmembers,year);
            }
        }
    }

    fn timber_truth(&self)->Option<crate::materials::MaterialProperties> {
        self.ground_truth.get(1).map(|m|m.properties)
    }

    fn stone_truth(&self)->Option<crate::materials::MaterialProperties> {
        self.ground_truth.get(2).map(|m|m.properties)
    }

    /// Prototyping: experimenting residents with material may draft tool or
    /// float designs. Physics tests them; successes enter the household
    /// repertoire, failures are logged as evidence. No water-raft scripts:
    /// drafts are probabilistic, costly, and can fail.
    fn step_experimentation(&mut self, days:f32) {
        let year=self.year; let seed=self.seed;
        let timber=self.timber_truth(); let stone=self.stone_truth();
        let experimenters:Vec<(u64,Position,Option<u64>,f32)>=self.residents.iter()
            .filter(|r|r.health>0.0&&r.current_action==ActionPrimitive::Experiment)
            .map(|r|(r.id,r.position,r.life.kinship.household,r.practice.skill(ActionPrimitive::Experiment))).collect();
        for (rid,pos,household,skill) in experimenters {
            // Nearest workable feature decides what gets prototyped.
            let mut best:Option<(FeatureKind,f32)>=None;
            for f in &self.features {
                if !matches!(f.kind,FeatureKind::DeepWater|FeatureKind::LooseMaterial|FeatureKind::RockFace){continue;}
                let d=dist(pos,f.position);
                if d<40.0&&best.map(|(_,bd)|d<bd).unwrap_or(true){best=Some((f.kind,d));}
            }
            let Some((kind,_))=best else{continue;};
            let Some(hid)=household else{continue;};
            let stock=self.households.iter().find(|h|h.id==hid).map(|h|h.shared_material).unwrap_or(0.0);
            if stock<1.0{continue;}
            let roll=stream_unit(seed,rid.wrapping_mul(7_345_345).wrapping_add(year.to_bits()));
            if roll>=(0.005+skill*0.015)*days.min(2.0){continue;}
            let id=self.next_id; self.next_id+=1;
            let variation=stream_unit(seed,id.wrapping_mul(999983))*2.0-1.0;
            let (genome,truth,function_label)=match kind {
                FeatureKind::DeepWater=>match timber {
                    Some(t)=>(seed_float_design(id,skill,variation),t,"float"),
                    None=>continue,
                },
                _=>match stone {
                    Some(t)=>{
                        let f=match stream_unit(seed,id.wrapping_mul(13)) {
                            x if x<0.34=>Function::Cut,
                            x if x<0.67=>Function::Pierce,
                            _=>Function::Impact,
                        };
                        (seed_tool_design(id,f,skill,variation),t,"tool")
                    }
                    None=>continue,
                },
            };
            if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                h.shared_material=(h.shared_material-2.0).max(0.0);
            }
            let draft=test_prototype(&genome,truth,skill);
            if draft.result.success {
                self.draft_count+=1;
                if let Some(h)=self.households.iter_mut().find(|h|h.id==hid) {
                    h.remember_design(genome);
                }
                if let Some(r)=self.residents.iter_mut().find(|r|r.id==rid) {
                    r.knowledge.learn(format!("design::{function_label}"),draft.result.actual,0.5);
                }
                self.causal_log.push(year,CausalNode::Outcome{resident_id:Some(rid),
                    label:format!("household {hid} devised a working {function_label} (surprise {:.2})",draft.result.surprise),value:draft.result.actual});
            } else if draft.result.surprise>0.4 {
                self.causal_log.push(year,CausalNode::Outcome{resident_id:Some(rid),
                    label:format!("failed {function_label} prototype taught household {hid}"),value:-draft.result.surprise});
            }
        }
    }

    fn household_craft(&self,hid:u64)->Option<Craft> {
        let timber=self.timber_truth()?;
        let repertoire:Vec<DesignGenome>=self.households.iter().find(|h|h.id==hid)
            .map(|h|h.designs.clone()).unwrap_or_default();
        // Household repertoire plus settlement-built float designs members have seen.
        let mut designs=repertoire;
        for s in &self.structures {
            designs.push(s.design.clone());
        }
        craft_from_designs(designs.iter(),&timber)
    }

    /// Warbands form, fight with invented weapons, remember outcomes.
    /// Cowardly math: outmatched bands break instead of dying pointlessly.
    fn step_warfare(&mut self, days:f32) {
        let year=self.year; let seed=self.seed;
        let attackers:Vec<(u64,f32,f32,Option<u64>)>=self.residents.iter()
            .filter(|r|r.health>0.0&&r.current_action==ActionPrimitive::Attack)
            .map(|r|(r.id,r.position.x,r.position.y,r.life.kinship.household)).collect();
        let live_monsters:Vec<(u64,f32,f32)>=self.monsters.iter()
            .filter(|m|m.health>0.0).map(|m|(m.id,m.position.x,m.position.y)).collect();
        // Re-group every step; morale persists by membership overlap.
        let groups=form_warbands(&attackers,&live_monsters,60.0);
        let mut next_bands:Vec<Warband>=Vec::new();
        for (members,mid) in groups {
            let sid=self.settlements.iter()
                .filter(|s|members.iter().any(|id|s.members.contains(id)))
                .max_by_key(|s|members.iter().filter(|id|s.members.contains(id)).count())
                .map(|s|s.id);
            let reuse=self.warbands.iter().find(|b|b.target_monster==Some(mid)
                &&b.members.iter().any(|id|members.contains(id)));
            let mut band=match reuse {
                Some(old)=>{
                    let mut b=old.clone(); b.members=members.clone(); b.target_monster=Some(mid);
                    if sid.is_some(){b.settlement_id=sid;}
                    b
                }
                None=>{
                    let id=self.next_id; self.next_id+=1;
                    let bonus=sid.map(|s|coordination_bonus(&self.institutions,s,ActionPrimitive::Attack)).unwrap_or(0.0);
                    Warband{id,settlement_id:sid,members:members.clone(),morale:(0.5+bonus).min(1.0),target_monster:Some(mid),battles:0}
                }
            };
            // Resolve pitched battles on some steps, not every heartbeat.
            let roll=stream_unit(seed,band.id.wrapping_mul(5_647_847).wrapping_add(year.to_bits()));
            if roll<0.30*days.min(2.0) {
                let melee=sid.and_then(|s|{
                    let designs:Vec<DesignGenome>=self.households.iter()
                        .filter(|h|self.settlements.iter().find(|x|x.id==s)
                            .map(|x|dist(h.home,x.center)<55.0).unwrap_or(false))
                        .flat_map(|h|h.designs.clone()).collect();
                    self.timber_truth().map(|t|melee_from_repertoire(&designs,&t))
                }).unwrap_or(0.0);
                let cx=members.iter().filter_map(|id|self.residents.iter().find(|r|r.id==*id))
                    .map(|r|r.position.x).sum::<f32>()/members.len().max(1) as f32;
                let cy=members.iter().filter_map(|id|self.residents.iter().find(|r|r.id==*id))
                    .map(|r|r.position.y).sum::<f32>()/members.len().max(1) as f32;
                let armor=0.10+0.30*self.structures.iter()
                    .filter(|s|s.integrity>0.2&&dist(s.position,Position{x:cx,y:cy})<40.0)
                    .map(|s|s.capabilities().defense).fold(0.0_f32,f32::max);
                let force=assemble_force(&band,melee,armor);
                if let Some(mon)=self.monsters.iter().find(|m|m.id==mid) {
                    let species=crate::monsters::MonsterSpecies{id:0,
                        body_mass_kg:mon.archetype.body_mass_kg,speed_m_s:mon.archetype.speed*20.0,
                        armor:mon.archetype.armor,bite_force:mon.archetype.aggression,
                        aggression:mon.archetype.aggression,intelligence:mon.archetype.intelligence,
                        reproduction_rate:mon.archetype.reproduction,food_need_kg_day:mon.archetype.body_mass_kg*0.02};
                    let outcome=resolve_battle(force,&species,1.0);
                    let per_member=outcome.human_losses/members.len().max(1) as f32;
                    for id in &members {
                        if let Some(r)=self.residents.iter_mut().find(|r|r.id==*id) {
                            r.health=(r.health-per_member).max(0.0);
                        }
                    }
                    if let Some(m)=self.monsters.iter_mut().find(|m|m.id==mid) {
                        m.health=(m.health-outcome.monster_losses*0.5).max(0.0);
                        if outcome.monsters_repulsed {
                            m.position.x+=(m.position.x-cx).signum()*20.0;
                            m.position.y+=(m.position.y-cy).signum()*20.0;
                            self.monsters_repelled+=1;
                        }
                    }
                    for s in self.structures.iter_mut().filter(|s|dist(s.position,Position{x:cx,y:cy})<30.0) {
                        s.integrity=(s.integrity-outcome.settlement_damage*0.5).max(0.0);
                    }
                    update_morale(&mut band,&outcome);
                    self.battles+=1;
                    if outcome.monsters_repulsed {
                        self.causal_log.push(year,CausalNode::Outcome{resident_id:None,
                            label:format!("warband {} repulsed monster {mid}",band.id),value:0.8});
                    }
                }
            }
            if !disbanded(&band){next_bands.push(band);}
        }
        self.warbands=next_bands;
        // Campaign supply: fighting far from home eats extra.
        for b in &self.warbands {
            let home=b.settlement_id.and_then(|sid|self.settlements.iter().find(|s|s.id==sid).map(|s|s.center));
            for id in &b.members {
                let (pos,hhome)=match self.residents.iter().find(|r|r.id==*id) {
                    Some(r)=>(r.position,r.life.kinship.household.and_then(|hid|self.households.iter().find(|h|h.id==hid).map(|h|h.home))),
                    None=>continue,
                };
                let anchor=home.or(hhome).unwrap_or(pos);
                let d=dist(pos,anchor);
                let drain=supply_drain(d,days);
                if drain>0.0 {
                    if let Some(r)=self.residents.iter_mut().find(|r|r.id==*id) {
                        r.mind.needs.hunger=(r.mind.needs.hunger+drain).max(0.0).min(1.0);
                    }
                }
            }
        }
    }
}

fn move_per_household(moved:f32,takers:usize)->f32 {
    moved/takers.max(1) as f32
}

fn stream_unit(seed:u64,salt:u64)->f32 {
    let mut x=seed ^ salt.wrapping_mul(0x9E3779B97F4A7C15);
    x^=x>>30; x=x.wrapping_mul(0xBF58476D1CE4E5B9);
    x^=x>>27; x=x.wrapping_mul(0x94D049BB133111EB);
    x^=x>>31;
    (x as f64/u64::MAX as f64) as f32
}
