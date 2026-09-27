use crate::world::Position;

#[derive(Clone, Debug)]
pub struct SettlementCluster {
    pub member_indices:Vec<usize>,
    pub center:Position,
}

fn distance(a:Position,b:Position)->f32 {((a.x-b.x).powi(2)+(a.y-b.y).powi(2)).sqrt()}

pub fn detect_settlements(points:&[Position],link_distance:f32,min_members:usize)->Vec<SettlementCluster> {
    let mut visited=vec![false;points.len()];let mut out=Vec::new();
    for i in 0..points.len() {
        if visited[i]{continue;} let mut stack=vec![i];let mut members=Vec::new();visited[i]=true;
        while let Some(a)=stack.pop(){members.push(a);for b in 0..points.len(){if !visited[b]&&distance(points[a],points[b])<=link_distance{visited[b]=true;stack.push(b);}}}
        if members.len()>=min_members {
            let (sx,sy)=members.iter().fold((0.0,0.0),|(x,y),&j|(x+points[j].x,y+points[j].y));
            out.push(SettlementCluster{center:Position{x:sx/members.len() as f32,y:sy/members.len() as f32},member_indices:members});
        }
    } out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn distant_groups_form_two_settlements() {
        let p=[Position{x:0.0,y:0.0},Position{x:1.0,y:0.0},Position{x:100.0,y:0.0},Position{x:101.0,y:0.0}];
        assert_eq!(detect_settlements(&p,5.0,2).len(),2);
    }
}
