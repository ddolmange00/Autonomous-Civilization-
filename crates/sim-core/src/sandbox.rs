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
    social_dynamics::spend_time,
    generation::{annual_mortality_risk,conception_propensity,ReproductionContext},
    demography::{HeritableTraits,inherit},
    households::Household,
    life_history::inherit_personality,
    causal_log::{CausalLog, CausalNode},
    world::Position,
};

#[derive(Clone, Debug)]
pub struct ActionScore { pub action: ActionPrimitive, pub score: f32 }

#[derive(Clone, Debug)]
pub struct Resident {
    pub id:u64, pub position:Position, pub mind:AgentMind, pub memory:EpisodicMemory,
    pub health:f32, pub current_action:ActionPrimitive, pub top_scores:Vec<ActionScore>, pub awareness:Awareness,
    pub life:LifeHistory, pub knowledge:KnowledgeStore,
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
}

#[derive(Clone, Debug)]
pub struct Sandbox {
    pub seed:u64, pub year:f64, pub residents:Vec<Resident>, pub animals:Vec<SandboxAnimal>, pub monsters:Vec<SandboxMonster>,
    pub features:Vec<SandboxFeature>, pub households:Vec<Household>, pub events:Vec<WorldEvent>, pub causal_log:CausalLog, pub next_id:u64,
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
fn move_toward(p:&mut Position,target:Position,amount:f32) {
    let dx=target.x-p.x; let dy=target.y-p.y; let d=(dx*dx+dy*dy).sqrt();
    if d>0.001 { let s=amount.min(d)/d; p.x+=dx*s; p.y+=dy*s; }
}
fn move_away(p:&mut Position,target:Position,amount:f32) {
    let dx=p.x-target.x; let dy=p.y-target.y; let d=(dx*dx+dy*dy).sqrt();
    if d>0.001 { p.x+=dx/d*amount; p.y+=dy/d*amount; }
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
                needs:Needs{hunger:0.25+unit(seed,i*20+11)*0.25,safety:0.2,rest:0.15,belonging:0.3,status:0.2,curiosity:0.2+unit(seed,i*20+12)*0.5,care:0.25},
                ..Default::default()
            };
            residents.push(Resident{
                id:i+1, position:Position{x:-120.0+signed(seed,i*20+13)*55.0,y:signed(seed,i*20+14)*90.0},
                mind,memory:EpisodicMemory{episodes:vec![],capacity:64},health:1.0,
                current_action:ActionPrimitive::Observe,top_scores:vec![],awareness:Awareness::default(),life:LifeHistory{birth_year:-18.0-unit(seed,i*20+15) as f64*28.0,sex:if unit(seed,i*20+22)<0.5{Sex::Female}else{Sex::Male},last_birth_year:None,biological:HeritableTraits{stature:unit(seed,i*20+16),body_mass:unit(seed,i*20+17),cold_tolerance:unit(seed,i*20+18),heat_tolerance:unit(seed,i*20+19),pigmentation:unit(seed,i*20+20),disease_resistance:unit(seed,i*20+21)},kinship:Kinship::default(),social:SocialMemory::default()},knowledge:KnowledgeStore::default(),
            });
        }
        let mut features=Vec::new(); let mut id=10_000u64;
        for i in 0..42u64 {
            features.push(SandboxFeature{id,kind:FeatureKind::Vegetation,
                position:Position{x:-220.0+unit(seed,500+i)*440.0,y:-160.0+unit(seed,700+i)*320.0},
                danger:0.02,food:0.12+unit(seed,900+i)*0.25,material:0.5}); id+=1;
        }
        for i in 0..18u64 {
            features.push(SandboxFeature{id,kind:FeatureKind::RockFace,
                position:Position{x:70.0+unit(seed,1100+i)*180.0,y:-180.0+unit(seed,1300+i)*360.0},
                danger:0.08,food:0.0,material:0.8}); id+=1;
        }
        for y in -9..=9 {
            features.push(SandboxFeature{id,kind:FeatureKind::DeepWater,
                position:Position{x:0.0,y:y as f32*22.0},danger:0.65,food:0.0,material:0.0}); id+=1;
        }
        Self{seed,year:0.0,residents,animals:vec![],monsters:vec![],features,households:vec![],events:vec![],causal_log:CausalLog{nodes:vec![],capacity:2048},next_id:id}
    }

    pub fn spawn_resident_at(&mut self, position:Position) {
        let id=self.next_id; self.next_id+=1;
        let s=self.seed^id;
        let mind=AgentMind{
            traits:Traits{threat_sensitivity:unit(s,1),aggression:unit(s,2),curiosity:unit(s,3),empathy:unit(s,4),
                conformity:unit(s,5),persistence:unit(s,6),risk_tolerance:unit(s,7),novelty_seeking:unit(s,8),
                social_trust:unit(s,9),planning_horizon:unit(s,10)},
            needs:Needs{hunger:0.25,safety:0.2,rest:0.15,belonging:0.3,status:0.2,curiosity:0.35,care:0.25},
            ..Default::default()
        };
        self.residents.push(Resident{id,position,mind,memory:EpisodicMemory{episodes:vec![],capacity:64},health:1.0,
            current_action:ActionPrimitive::Observe,top_scores:vec![],awareness:Awareness::default(),life:LifeHistory{birth_year:self.year-18.0-unit(s,11) as f64*22.0,sex:if unit(s,18)<0.5{Sex::Female}else{Sex::Male},last_birth_year:None,biological:HeritableTraits{stature:unit(s,12),body_mass:unit(s,13),cold_tolerance:unit(s,14),heat_tolerance:unit(s,15),pigmentation:unit(s,16),disease_resistance:unit(s,17)},kinship:Kinship::default(),social:SocialMemory::default()},knowledge:KnowledgeStore::default(),});
        self.causal_log.push(self.year,CausalNode::WorldEvent{event_id:id,label:"Resident spawned".into()});
    }
    pub fn spawn_animal_at(&mut self, position:Position) { self.spawn_animal_with(position,AnimalArchetype::default()); }
    pub fn spawn_animal_with(&mut self, position:Position, archetype:AnimalArchetype) {
        let id=self.next_id;self.next_id+=1;
        self.animals.push(SandboxAnimal{id,position,hunger:0.35,health:1.0,archetype});
        self.causal_log.push(self.year,CausalNode::WorldEvent{event_id:id,label:"Animal spawned".into()});
    }
    pub fn grow_vegetation_at(&mut self, position:Position, count:u32) {
        for i in 0..count {
            let id=self.next_id;self.next_id+=1;
            let a=unit(self.seed,id*3)*std::f32::consts::TAU;let r=unit(self.seed,id*3+1)*32.0;
            self.features.push(SandboxFeature{id,kind:FeatureKind::Vegetation,position:Position{x:position.x+a.cos()*r,y:position.y+a.sin()*r},
                danger:0.02,food:0.12+unit(self.seed,id*3+2)*0.25,material:0.5});
        }
    }
    pub fn deposit_minerals_at(&mut self, position:Position, count:u32) {
        for i in 0..count {
            let id=self.next_id;self.next_id+=1;
            let a=unit(self.seed,id*5)*std::f32::consts::TAU;let r=unit(self.seed,id*5+1)*24.0;
            self.features.push(SandboxFeature{id,kind:FeatureKind::LooseMaterial,position:Position{x:position.x+a.cos()*r,y:position.y+a.sin()*r},
                danger:0.01,food:0.0,material:0.65+unit(self.seed,id*5+2)*0.3});
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
        let snapshot_monsters=self.monsters.clone();
        let snapshot_animals=self.animals.clone();
        let active_events:Vec<_>=self.events.iter().copied().filter(|e|e.active(self.year)).collect();
        let features=self.features.clone();
        for r in &mut self.residents {
            if r.health<=0.0 { continue; }
            r.mind.needs.hunger=(r.mind.needs.hunger+days*0.006).clamp(0.0,1.0);
            r.mind.needs.rest=(r.mind.needs.rest+days*0.002).clamp(0.0,1.0);
            let mut perceived=Vec::new();
            for e in &active_events {
                let influence=e.influence_at(r.position);
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
                    perceived.push(PerceivedFeature{id:f.id,kind:f.kind,distance_m:d,danger:f.danger,
                        food_hint:f.food,material_hint:f.material,uncertainty:(d/120.0).clamp(0.05,0.8)});
                }
            }
            for a in &snapshot_animals {
                let d=dist(r.position,a.position);
                if a.health>0.0 && d<=75.0 { perceived.push(PerceivedFeature{id:a.id,kind:FeatureKind::Creature,distance_m:d,danger:0.08+a.fear*0.08,food_hint:0.35,material_hint:0.18,uncertainty:(d/100.0).clamp(0.05,0.7)}); }
            }
            for m in &snapshot_monsters {
                let d=dist(r.position,m.position);
                if m.health>0.0 && d<=110.0 {
                    let confidence=(1.0-d/140.0).clamp(0.1,1.0);
                    r.awareness.observe(SituationReport{kind:SituationKind::CreatureThreat,source_id:Some(m.id),perceived_severity:(m.archetype.aggression*m.health*(0.5+m.archetype.body_mass_kg.sqrt()/60.0)).clamp(0.0,1.5),confidence,observed_year:self.year,location:[m.position.x,m.position.y]});
                    perceived.push(PerceivedFeature{id:m.id,kind:FeatureKind::Creature,distance_m:d,
                        danger:(m.archetype.aggression*m.health*(0.5+m.archetype.body_mass_kg.sqrt()/60.0)).clamp(0.0,1.5),food_hint:0.0,material_hint:0.25,uncertainty:(d/140.0).clamp(0.05,0.75)});
                }
            }
            let affordances=generate(&perceived,LocalCapabilities{reach_m:12.0,cutting:0.05,digging:0.04,carrying:0.2,heat_tolerance:0.0});
            if affordances.is_empty() { continue; }
            let mut scored:Vec<(usize,f32)>=affordances.iter().enumerate().map(|(i,a)|{
                let mem=r.memory.recalled_value(a.action,self.year);
                let noise=signed(self.seed,r.id.wrapping_mul(100_000)+self.year.to_bits()+i as u64)*0.08;
                (i,r.mind.score(a,noise)+mem*0.25)
            }).collect();
            scored.sort_by(|a,b|b.1.total_cmp(&a.1));
            r.top_scores=scored.iter().take(5).map(|(i,s)|ActionScore{action:affordances[*i].action,score:*s}).collect();
            let chosen=&affordances[scored[0].0]; r.current_action=chosen.action;
            self.causal_log.push(self.year,CausalNode::Decision{resident_id:r.id,action:chosen.action,score:scored[0].1});
            let target=chosen.target.and_then(|id|{
                features.iter().find(|f|f.id==id).map(|f|f.position)
                    .or_else(||snapshot_monsters.iter().find(|m|m.id==id).map(|m|m.position))
            });
            if let Some(t)=target {
                match chosen.action {
                    ActionPrimitive::Avoid|ActionPrimitive::Hide=>move_away(&mut r.position,t,days*1.4),
                    ActionPrimitive::Attack=>move_toward(&mut r.position,t,days*1.8),
                    ActionPrimitive::Gather|ActionPrimitive::Carry|ActionPrimitive::Observe|ActionPrimitive::Experiment|
                    ActionPrimitive::Dig|ActionPrimitive::Strike|ActionPrimitive::Cut|ActionPrimitive::Bind=>move_toward(&mut r.position,t,days*0.7),
                    _=>{}
                }
            }
            let value=if chosen.action==ActionPrimitive::Avoid {chosen.expected.safety} else {chosen.expected.food+chosen.expected.knowledge+chosen.expected.status-chosen.expected.physical_risk};
            r.mind.learn_action(chosen.action,value,0.04);
            if matches!(chosen.action,ActionPrimitive::Observe|ActionPrimitive::Experiment|ActionPrimitive::Strike|ActionPrimitive::Cut|ActionPrimitive::Dig) {
                r.knowledge.learn(format!("action::{:?}",chosen.action),value,0.18+chosen.expected.knowledge*0.5);
            }
            r.memory.remember(Episode{year:self.year,action:chosen.action,target:chosen.target,value,surprise:chosen.uncertainty,
                danger:chosen.expected.physical_risk,social_visibility:0.2});
        }
        // Local teaching: nearby residents can pass imperfect knowledge according to trust.
        let teachers:Vec<_>=self.residents.iter().filter(|r|r.health>0.0&&!r.knowledge.items.is_empty())
            .map(|r|(r.id,r.position,r.knowledge.clone())).collect();
        for r in &mut self.residents {
            for (teacher,pos,knowledge) in &teachers {
                if *teacher==r.id {continue;} let d=dist(r.position,*pos);
                if d<=16.0 && r.mind.traits.social_trust>0.35 {
                    let trust=(0.25+r.mind.traits.social_trust*0.65).clamp(0.0,1.0);
                    let distortion=signed(self.seed,r.id.wrapping_mul(911_003)+*teacher)*0.12;
                    knowledge.transmit_to(&mut r.knowledge,trust,distortion);
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
                    r.awareness.hear(*report,trust,distortion,self.year);
                    self.causal_log.push(self.year,CausalNode::Transmission{from:*source,to:r.id,label:format!("{:?} rumor",report.kind)});
                }
            }
        }
        self.events.retain(|e|e.active(self.year));
        for a in &mut self.animals {
            if a.health<=0.0 {continue;} a.hunger=(a.hunger+days*0.002).clamp(0.0,1.0);
            let nearest_monster=self.monsters.iter().filter(|m|m.health>0.0).map(|m|(m.position,dist(a.position,m.position))).min_by(|x,y|x.1.total_cmp(&y.1));
            if let Some((p,d))=nearest_monster {if d<70.0 {move_away(&mut a.position,p,days*(0.35+a.archetype.speed+a.archetype.fear*0.45));continue;}}
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
                    self.residents[idx].health=(self.residents[idx].health-damage).max(0.0);
                    m.hunger=(m.hunger-damage*1.5).max(0.0);
                } else if d<180.0 && m.hunger>0.35 {
                    m.motion=if m.archetype.speed>0.9{MotionState::Run}else{MotionState::Walk};
                    move_toward(&mut m.position,target,days*(0.45+m.archetype.speed+m.archetype.aggression*0.35));
                } else {
                    m.position.x+=signed(self.seed,self.year.to_bits()+m.id)*days*0.4;
                    m.position.y+=signed(self.seed,self.year.to_bits()+m.id+1)*days*0.4;
                }
            }
        }
        for r in &mut self.residents {
            if r.current_action==ActionPrimitive::Attack {
                for m in &mut self.monsters {
                    if m.health>0.0 && dist(r.position,m.position)<9.0 {
                        m.health=(m.health-(0.01+0.025*r.mind.traits.aggression)*days).max(0.0);
                    }
                }
            }
        }
    }
}
