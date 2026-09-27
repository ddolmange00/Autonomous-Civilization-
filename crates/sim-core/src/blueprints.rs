use crate::species::MonsterArchetype;
use serde::{Deserialize, Serialize};

pub const MAX_PIXEL_SIDE:u8=32;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct PixelCell {
    pub filled:bool,
    pub palette:u8,
    pub emissive:bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PixelSkin {
    pub width:u8,
    pub height:u8,
    pub pixels:Vec<PixelCell>,
}

impl PixelSkin {
    pub fn new(width:u8,height:u8)->Self {
        let w=width.clamp(1,MAX_PIXEL_SIDE);let h=height.clamp(1,MAX_PIXEL_SIDE);
        Self{width:w,height:h,pixels:vec![PixelCell::default();w as usize*h as usize]}
    }
    pub fn index(&self,x:u8,y:u8)->Option<usize> {
        if x<self.width&&y<self.height {Some(y as usize*self.width as usize+x as usize)} else {None}
    }
    pub fn set(&mut self,x:u8,y:u8,cell:PixelCell) {if let Some(i)=self.index(x,y){self.pixels[i]=cell;}}
    pub fn filled_fraction(&self)->f32 {
        self.pixels.iter().filter(|p|p.filled).count() as f32/self.pixels.len().max(1) as f32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonsterBlueprint {
    pub id:u64,
    pub name:String,
    pub skin:PixelSkin,
    pub archetype:MonsterArchetype,
    pub scale:f32,
}

impl MonsterBlueprint {
    pub fn effective_mass_kg(&self)->f32 {
        self.archetype.body_mass_kg*self.scale.max(0.1).powi(3)
    }
    pub fn visual_coverage(&self)->f32 {self.skin.filled_fraction()}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn pixel_skin_bounds_are_safe() {
        let mut s=PixelSkin::new(16,16);s.set(15,15,PixelCell{filled:true,..Default::default()});
        assert!(s.pixels[s.index(15,15).unwrap()].filled);assert!(s.index(16,0).is_none());
    }
    #[test] fn scaling_body_changes_mass_cubically() {
        let b=MonsterBlueprint{id:1,name:"x".into(),skin:PixelSkin::new(16,16),archetype:MonsterArchetype::default(),scale:2.0};
        assert!((b.effective_mass_kg()-b.archetype.body_mass_kg*8.0).abs()<0.01);
    }
}
