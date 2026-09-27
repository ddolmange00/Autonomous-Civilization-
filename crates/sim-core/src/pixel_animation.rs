use serde::{Deserialize, Serialize};
use crate::blueprints::{AnchorKind,MonsterBlueprint};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionState { Idle, Walk, Run, Attack, Hit, Death }

#[derive(Clone, Copy, Debug, Default)]
pub struct PixelTransform {
    pub offset_x:f32,
    pub offset_y:f32,
    pub scale_x:f32,
    pub scale_y:f32,
    pub shear_x:f32,
    pub rotation:f32,
}

pub fn body_transform(state:MotionState, phase:f32, speed:f32)->PixelTransform {
    let p=phase*std::f32::consts::TAU;
    match state {
        MotionState::Idle=>PixelTransform{offset_y:p.sin()*0.25,scale_x:1.0-p.sin()*0.01,scale_y:1.0+p.sin()*0.015,..Default::default()},
        MotionState::Walk=>PixelTransform{offset_y:p.sin().abs()*0.45,scale_x:1.0,scale_y:1.0,shear_x:p.sin()*0.025*speed,..Default::default()},
        MotionState::Run=>PixelTransform{offset_y:p.sin().abs()*0.75,scale_x:1.0+p.cos()*0.02,scale_y:1.0-p.cos()*0.02,shear_x:p.sin()*0.06*speed,rotation:p.sin()*0.025,..Default::default()},
        MotionState::Attack=>PixelTransform{offset_x:(p.sin().max(0.0))*1.4,scale_x:1.0+p.sin().max(0.0)*0.08,scale_y:1.0-p.sin().max(0.0)*0.05,shear_x:0.08,..Default::default()},
        MotionState::Hit=>PixelTransform{offset_x:-1.2*(1.0-phase.clamp(0.0,1.0)),rotation:-0.08*(1.0-phase.clamp(0.0,1.0)),scale_x:0.96,scale_y:1.04,..Default::default()},
        MotionState::Death=>PixelTransform{offset_y:-phase.clamp(0.0,1.0)*1.8,scale_x:1.0+phase*0.35,scale_y:(1.0-phase*0.75).max(0.15),rotation:phase*0.9,..Default::default()},
    }
}

pub fn pixel_offset(x:u8,y:u8,width:u8,height:u8,state:MotionState,phase:f32)->(f32,f32) {
    let nx=(x as f32/(width.max(2)-1) as f32)*2.0-1.0;
    let ny=(y as f32/(height.max(2)-1) as f32)*2.0-1.0;
    let p=phase*std::f32::consts::TAU;
    match state {
        MotionState::Walk|MotionState::Run=>{
            let lower=(ny.max(0.0)).powi(2);
            let stride=p.sin()*lower*(0.35+0.25*nx.signum());
            (stride,0.0)
        }
        MotionState::Attack=>{
            let front=(nx.max(0.0)).powi(2);
            (front*p.sin().max(0.0)*0.8,-front*p.sin().max(0.0)*0.12)
        }
        MotionState::Hit=>(-0.2*(1.0-ny.abs()),0.0),
        _=>(0.0,0.0),
    }
}

pub fn anchored_pixel_offset(blueprint:&MonsterBlueprint,x:u8,y:u8,state:MotionState,phase:f32)->(f32,f32) {
    let mut out=pixel_offset(x,y,blueprint.skin.width,blueprint.skin.height,state,phase);
    let p=phase*std::f32::consts::TAU;
    for a in &blueprint.anchors {
        let dx=x as f32-a.x as f32;let dy=y as f32-a.y as f32;let d2=dx*dx+dy*dy;
        if d2>9.0 {continue;} let influence=(1.0-d2/10.0).max(0.0);
        match (a.kind,state) {
            (AnchorKind::Foot,MotionState::Walk)|(AnchorKind::Foot,MotionState::Run)=>{out.0+=p.sin()*0.8*influence;out.1+=p.cos().min(0.0).abs()*0.25*influence;}
            (AnchorKind::Tail,MotionState::Idle)|(AnchorKind::Tail,MotionState::Walk)|(AnchorKind::Tail,MotionState::Run)=>{out.0+=(p*1.4).sin()*0.65*influence;}
            (AnchorKind::Head,MotionState::Attack)=>{out.0+=p.sin().max(0.0)*0.65*influence;}
            (AnchorKind::Attack,MotionState::Attack)=>{out.0+=p.sin().max(0.0)*1.25*influence;}
            _=>{}
        }
    } out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn death_collapses_vertical_scale(){assert!(body_transform(MotionState::Death,1.0,1.0).scale_y<body_transform(MotionState::Death,0.0,1.0).scale_y);}
    #[test] fn idle_is_subtle(){let t=body_transform(MotionState::Idle,0.25,1.0);assert!(t.offset_y.abs()<0.5);}
}
