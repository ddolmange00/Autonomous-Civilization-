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
        self.step_social_generation(days);
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
                    self.households.push(Household{id:hid,members:vec![a.id,b.id],home:Position{x:(a.position.x+b.position.x)*0.5,y:(a.position.y+b.position.y)*0.5},stored_food:80.0,shared_material:0.0,cohesion:0.55,migration_goal:None});
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
            let mut knowledge=KnowledgeStore::default();
            mother.knowledge.transmit_to(&mut knowledge,0.35,signed(s,23)*0.08);
            father.knowledge.transmit_to(&mut knowledge,0.35,signed(s,24)*0.08);
            self.residents.push(Resident{id,position,mind:AgentMind{traits,needs:Needs{hunger:0.15,safety:0.35,rest:0.35,belonging:0.65,status:0.0,curiosity:0.35,care:0.0},..Default::default()},
                memory:EpisodicMemory{episodes:vec![],capacity:64},health:1.0,current_action:ActionPrimitive::Observe,top_scores:vec![],awareness:Awareness::default(),
                life:LifeHistory{birth_year:year,sex:if unit(s,25)<0.5{Sex::Female}else{Sex::Male},last_birth_year:None,biological,
                    kinship:Kinship{parents:vec![mother.id,father.id],children:vec![],partners:vec![],household},social:SocialMemory::default()},knowledge});
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
    }

}
