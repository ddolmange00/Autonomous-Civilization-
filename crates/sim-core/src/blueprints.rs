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
    pub fn resize_nearest(&self,width:u8,height:u8)->Self {
        let mut out=Self::new(width,height);
        for y in 0..out.height {for x in 0..out.width {
            let sx=((x as f32+0.5)*self.width as f32/out.width as f32).floor().clamp(0.0,self.width as f32-1.0) as u8;
            let sy=((y as f32+0.5)*self.height as f32/out.height as f32).floor().clamp(0.0,self.height as f32-1.0) as u8;
            if let (Some(si),Some(di))=(self.index(sx,sy),out.index(x,y)){out.pixels[di]=self.pixels[si];}
        }} out
    }
    pub fn flood_fill(&mut self,x:u8,y:u8,replacement:PixelCell) {
        let Some(start)=self.index(x,y) else{return;}; let target=self.pixels[start];
        if target.filled==replacement.filled&&target.palette==replacement.palette&&target.emissive==replacement.emissive{return;}
        let mut stack=vec![(x,y)];
        while let Some((cx,cy))=stack.pop(){let Some(i)=self.index(cx,cy) else{continue;};let p=self.pixels[i];
            if p.filled!=target.filled||p.palette!=target.palette||p.emissive!=target.emissive{continue;}
            self.pixels[i]=replacement;
            if cx>0{stack.push((cx-1,cy));}if cy>0{stack.push((cx,cy-1));}if cx+1<self.width{stack.push((cx+1,cy));}if cy+1<self.height{stack.push((cx,cy+1));}
        }
    }
    pub fn filled_fraction(&self)->f32 {
        self.pixels.iter().filter(|p|p.filled).count() as f32/self.pixels.len().max(1) as f32
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorKind { Head, Eye, Foot, Tail, Attack }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct PixelAnchor { pub kind:AnchorKind, pub x:u8, pub y:u8 }

pub struct MonsterBlueprint {
    pub id:u64,
    pub name:String,
    pub skin:PixelSkin,
    pub archetype:MonsterArchetype,
    pub scale:f32,
    pub anchors:Vec<PixelAnchor>,
}

impl MonsterBlueprint {
    pub fn set_anchor(&mut self,anchor:PixelAnchor) {
        self.anchors.retain(|a|a.kind!=anchor.kind || a.kind==AnchorKind::Foot);
        if anchor.x<self.skin.width&&anchor.y<self.skin.height {self.anchors.push(anchor);}
    }
    pub fn anchors_of(&self,kind:AnchorKind)->impl Iterator<Item=&PixelAnchor>{self.anchors.iter().filter(move |a|a.kind==kind)}
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
    #[test] fn resize_preserves_basic_shape() {
        let mut s=PixelSkin::new(16,16);s.set(8,8,PixelCell{filled:true,palette:2,emissive:true});let r=s.resize_nearest(32,32);
        assert!(r.pixels.iter().any(|p|p.filled&&p.palette==2&&p.emissive));
    }
    #[test] fn flood_fill_changes_connected_region() {
        let mut s=PixelSkin::new(4,4);s.flood_fill(0,0,PixelCell{filled:true,palette:3,emissive:false});assert!(s.pixels.iter().all(|p|p.filled&&p.palette==3));
    }
    #[test] fn scaling_body_changes_mass_cubically() {
        let b=MonsterBlueprint{id:1,name:"x".into(),skin:PixelSkin::new(16,16),archetype:MonsterArchetype::default(),scale:2.0,anchors:vec![]};
        assert!((b.effective_mass_kg()-b.archetype.body_mass_kg*8.0).abs()<0.01);
    }
}
