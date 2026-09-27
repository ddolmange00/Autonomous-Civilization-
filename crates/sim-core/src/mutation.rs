use crate::design::DesignGenome;

pub fn mutate(parent:&DesignGenome, child_id:u64, noise:[f32;6])->DesignGenome {
    let n=|i:usize,scale:f32| 1.0+(noise[i].clamp(-1.0,1.0)*scale);
    let mut child=parent.clone();
    child.id=child_id;
    child.parent=Some(parent.id);
    child.generation=parent.generation+1;
    child.length_m=(parent.length_m*n(0,0.18)).max(0.03);
    child.width_m=(parent.width_m*n(1,0.20)).max(0.02);
    child.thickness_m=(parent.thickness_m*n(2,0.22)).max(0.005);
    child.curvature=(parent.curvature+noise[3]*0.12).clamp(0.,1.);
    child.edge_fraction=(parent.edge_fraction+noise[4]*0.10).clamp(0.,1.);
    child.binding_quality=(parent.binding_quality+noise[5]*0.10).clamp(0.,1.);
    child
}
