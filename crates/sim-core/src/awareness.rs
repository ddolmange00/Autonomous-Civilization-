use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SituationKind { CreatureThreat, Fire, Flood, FoodScarcity, WaterScarcity, Disease, UnknownPhenomenon }

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SituationReport {
    pub kind:SituationKind,
    pub source_id:Option<u64>,
    pub perceived_severity:f32,
    pub confidence:f32,
    pub observed_year:f64,
    pub location:[f32;2],
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Awareness {
    pub reports:BTreeMap<SituationKind,SituationReport>,
}

impl Awareness {
    pub fn observe(&mut self, report:SituationReport) {
        match self.reports.get(&report.kind) {
            Some(old) if old.confidence>report.confidence && old.observed_year>=report.observed_year => {}
            _ => { self.reports.insert(report.kind,report); }
        }
    }
    pub fn hear(&mut self, report:SituationReport, trust:f32, distortion:f32, now:f64) {
        let mut r=report;
        r.confidence=(r.confidence*trust.clamp(0.0,1.0)*(1.0-distortion.abs().clamp(0.0,0.9))).clamp(0.0,1.0);
        r.perceived_severity=(r.perceived_severity*(1.0+distortion.clamp(-0.8,1.5))).max(0.0);
        r.observed_year=now;
        self.observe(r);
    }
    pub fn confidence(&self,kind:SituationKind)->f32 {
        self.reports.get(&kind).map(|r|r.confidence).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn remote_threat_is_not_magically_known() {
        let a=Awareness::default();
        assert_eq!(a.confidence(SituationKind::CreatureThreat),0.0);
    }
    #[test] fn hearsay_can_distort_severity() {
        let base=SituationReport{kind:SituationKind::CreatureThreat,source_id:Some(9),perceived_severity:0.5,confidence:0.9,observed_year:1.0,location:[0.0,0.0]};
        let mut a=Awareness::default(); a.hear(base,0.8,0.5,1.1);
        assert!(a.reports[&SituationKind::CreatureThreat].perceived_severity>0.5);
        assert!(a.confidence(SituationKind::CreatureThreat)<0.9);
    }
}
