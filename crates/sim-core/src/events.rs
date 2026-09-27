use crate::{awareness::SituationKind, world::Position};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldEventKind { Fire, Flood, Earthquake, Storm, Drought, CreatureSpawn, ResourceDrop }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id:u64,
    pub kind:WorldEventKind,
    pub position:Position,
    pub radius:f32,
    pub intensity:f32,
    pub start_year:f64,
    pub duration_years:f64,
}

impl WorldEvent {
    pub fn active(&self,year:f64)->bool { year>=self.start_year && year<=self.start_year+self.duration_years }
    pub fn influence_at(&self,p:Position)->f32 {
        let dx=p.x-self.position.x; let dy=p.y-self.position.y;
        let d=(dx*dx+dy*dy).sqrt();
        if d>=self.radius {0.0} else {self.intensity*(1.0-d/self.radius.max(0.001))}
    }
    pub fn situation(&self)->Option<SituationKind> {
        match self.kind {
            WorldEventKind::Fire=>Some(SituationKind::Fire),
            WorldEventKind::Flood=>Some(SituationKind::Flood),
            WorldEventKind::Drought=>Some(SituationKind::WaterScarcity),
            _=>Some(SituationKind::UnknownPhenomenon),
        }
    }
}
