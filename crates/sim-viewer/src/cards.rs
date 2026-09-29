//! Player-facing cards for the selection panel: who someone is and what they
//! are doing in words, with a few bars, instead of raw per-tick numbers.
//! Raw decision scores appear only in debug mode (F3).
use sim_core::{
    agency::{ActionPrimitive, Traits},
    life_history::{LifeStage, Sex},
    sandbox::Resident,
    settlement_identity::SettlementIdentity,
};

use crate::{names, Selected, ViewerState};

pub fn activity(a: ActionPrimitive) -> &'static str {
    use ActionPrimitive::*;
    match a {
        Move => "걸어가는 중", Observe => "주변을 살피는 중", Gather => "먹을 것을 모으는 중", Carry => "짐을 나르는 중",
        Combine => "무언가를 조립하는 중", Separate => "무언가를 가르는 중", Bind => "엮고 묶는 중", Dig => "땅을 파는 중",
        Raise => "집을 짓는 중", Heat => "불을 다루는 중", Cool => "식히는 중", Strike => "두드리는 중",
        Pierce => "찌르는 중", Cut => "베는 중", Attack => "싸우는 중", Avoid => "피하는 중", Hide => "숨어 있는 중",
        Assist => "누군가를 돕는 중", Communicate => "이야기하는 중", Consume => "먹는 중", Rest => "쉬는 중",
        Experiment => "무언가를 시험하는 중",
    }
}

pub fn skill_name(a: ActionPrimitive) -> &'static str {
    use ActionPrimitive::*;
    match a {
        Gather => "채집", Cut => "벌목", Dig => "땅파기", Raise => "건축", Bind => "엮기", Carry => "운반",
        Strike => "두드리기", Pierce => "창술", Attack => "사냥·전투", Experiment => "실험", Heat => "불 다루기",
        Communicate => "말솜씨", Assist => "돌봄", Observe => "관찰", Avoid => "눈치", Hide => "숨기",
        Consume => "먹성", Rest => "느긋함", Move => "걸음", Combine => "조립", Separate => "가르기", Cool => "식히기",
    }
}

/// Actions that animate as working with a tool.
pub fn is_work(a: ActionPrimitive) -> bool {
    use ActionPrimitive::*;
    matches!(a, Gather | Cut | Dig | Strike | Pierce | Bind | Raise | Combine | Separate | Heat | Experiment)
}

fn bar(v: f32) -> String {
    let n = (v.clamp(0.0, 1.0) * 10.0).round() as usize;
    format!("{}{}", "■".repeat(n), "□".repeat(10 - n))
}

fn stage_name(s: LifeStage) -> &'static str {
    match s { LifeStage::Infant => "아기", LifeStage::Child => "어린이", LifeStage::Adolescent => "청소년", LifeStage::Adult => "어른", LifeStage::Elder => "노인" }
}

/// The three most pronounced personality traits, in words.
fn traits(t: &Traits) -> String {
    let all = [
        (t.curiosity, "호기심 많음", "무심함"), (t.persistence, "끈기 있음", "쉽게 포기함"),
        (t.threat_sensitivity, "겁이 많음", "대담함"), (t.aggression, "거침", "온순함"),
        (t.empathy, "다정함", "냉정함"), (t.conformity, "순응적", "독립적"),
        (t.risk_tolerance, "모험적", "신중함"), (t.novelty_seeking, "새것을 좋아함", "익숙한 것을 좋아함"),
        (t.social_trust, "잘 믿음", "의심 많음"), (t.planning_horizon, "계획적", "즉흥적"),
    ];
    let mut v: Vec<_> = all.iter().map(|&(x, hi, lo)| ((x - 0.5).abs(), if x >= 0.5 { hi } else { lo })).collect();
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    v.iter().take(3).map(|x| x.1).collect::<Vec<_>>().join(" · ")
}

fn village_of<'a>(state: &'a ViewerState, id: u64) -> Option<&'a SettlementIdentity> {
    state.sim.settlements.iter().find(|s| s.members.contains(&id))
}

pub fn resident_name(r: &Resident) -> String {
    names::person(r.id, r.life.kinship.household.unwrap_or(r.id))
}

fn resident_card(state: &ViewerState, r: &Resident) -> String {
    let year = state.sim.year;
    let sex = if r.life.sex == Sex::Female { "여" } else { "남" };
    let mut s = format!("{}\n{} · {:.0}살 {}\n", resident_name(r), sex, r.life.age(year), stage_name(r.life.stage(year)));
    let village = village_of(state, r.id).map(|v| format!("{} 마을", names::village(&v.lexicon))).unwrap_or_else(|| "떠돌이".into());
    let household = r.life.kinship.household.and_then(|h| state.sim.households.iter().find(|x| x.id == h));
    s.push_str(&match household {
        Some(h) => format!("{} · 가족 {}명\n\n", village, h.members.len()),
        None => format!("{} · 혼자\n\n", village),
    });
    if r.health <= 0.0 {
        s.push_str("세상을 떠났습니다.\n");
        return s;
    }
    s.push_str(&format!("지금: {}\n\n", activity(r.current_action)));
    s.push_str(&format!("체력   {}\n배고픔 {}\n불안   {}\n\n", bar(r.health), bar(r.mind.needs.hunger), bar(r.mind.needs.safety)));
    s.push_str(&format!("성격: {}\n", traits(&r.mind.traits)));
    let skills: Vec<_> = r.practice.dominant(2).into_iter().filter(|(_, x)| *x > 0.05).map(|(a, _)| skill_name(a)).collect();
    if !skills.is_empty() { s.push_str(&format!("재주: {}\n", skills.join(" · "))); }
    s.push_str(&format!("배우자 {} · 자녀 {} · 아는 것 {}가지", r.life.kinship.partners.len(), r.life.kinship.children.len(), r.knowledge.items.len()));
    if state.debug {
        s.push_str("\n\n[디버그] 행동 점수\n");
        for q in &r.top_scores { s.push_str(&format!("{:?} {:+.3}\n", q.action, q.score)); }
    }
    s
}

/// Korean name of a growth-tree era label (derived from what a tribe knows, never a gate).
pub fn era_name(era: &str) -> &str {
    match era {
        "stone-using" => "석기", "farming" => "농경", "copper-working" => "구리 가공", "bronze-casting" => "청동 주조",
        "iron-forging" => "철기", "steel-forging" => "강철", "gunpowder" => "화약", "steam" => "증기", "electric" => "전기",
        other => other,
    }
}

fn temperament(v: &SettlementIdentity) -> String {
    let p = v.profile();
    let mut words = Vec::new();
    if p.confrontation > p.avoidance + 0.05 { words.push("맞서 싸우는 편"); } else if p.avoidance > p.confrontation + 0.05 { words.push("위험을 피하는 편"); }
    if p.experimentation > 0.1 { words.push("새로운 시도를 즐김"); }
    if p.cooperation > 0.1 { words.push("서로 잘 도움"); }
    if p.construction > 0.1 { words.push("짓는 것을 중시함"); }
    if words.is_empty() { "아직 뚜렷한 기질이 없음".into() } else { words.join(" · ") }
}

fn village_card(state: &ViewerState, v: &SettlementIdentity) -> String {
    let mut s = format!("{} 마을\n{:.0}년에 세워짐", names::village(&v.lexicon), v.founded_year);
    if let Some(parent) = v.parent_id.and_then(|p| state.sim.settlements.iter().find(|x| x.id == p)) {
        s.push_str(&format!(" · {}에서 갈라져 나옴", names::village(&parent.lexicon)));
    }
    let alive = v.members.iter().filter(|id| state.sim.residents.iter().any(|r| r.id == **id && r.health > 0.0)).count();
    s.push_str(&format!("\n\n인구 {}명\n식량 비축 {:.0} · 자재 {:.0}\n마을이 아는 것 {}가지\n\n기질: {}\n", alive, v.shared_food, v.shared_material, v.knowledge_items, temperament(v)));
    let specs: Vec<_> = v.top_specializations(3).into_iter().filter(|(_, x)| *x > 0.02).map(|(a, _)| skill_name(a)).collect();
    if !specs.is_empty() { s.push_str(&format!("주로 하는 일: {}\n", specs.join(" · "))); }
    if let Some(era) = state.sim.growth.era(v.id) { s.push_str(&format!("시대: {}\n", era_name(era))); }
    if let Some(t) = state.sim.growth.tribes.get(&v.id) { s.push_str(&format!("발견한 것 {}가지\n", t.discoveries.len())); }
    s
}

fn group_card(state: &ViewerState, ids: &[u64]) -> String {
    let members: Vec<&Resident> = state.sim.residents.iter().filter(|r| r.health > 0.0 && ids.contains(&r.id)).collect();
    if members.is_empty() { return "선택한 사람들이 모두 사라졌습니다".into(); }
    let health = members.iter().map(|r| r.health).sum::<f32>() / members.len() as f32;
    let hunger = members.iter().map(|r| r.mind.needs.hunger).sum::<f32>() / members.len() as f32;
    let adults = members.iter().filter(|r| matches!(r.life.stage(state.sim.year), LifeStage::Adult | LifeStage::Elder)).count();
    let mut doing: Vec<(ActionPrimitive, usize)> = Vec::new();
    for r in &members {
        match doing.iter_mut().find(|(a, _)| *a == r.current_action) { Some(e) => e.1 += 1, None => doing.push((r.current_action, 1)) }
    }
    doing.sort_by(|a, b| b.1.cmp(&a.1));
    let mut s = format!("{}명 선택 (어른 {} · 아이 {})\n\n평균 체력 {}\n평균 배고픔 {}\n\n하는 일\n", members.len(), adults, members.len() - adults, bar(health), bar(hunger));
    for (a, n) in doing.iter().take(5) { s.push_str(&format!("  {} {}명\n", activity(*a), n)); }
    s
}

pub fn inspector_text(state: &ViewerState) -> String {
    match &state.selected {
        None => String::new(),
        Some(Selected::Resident(id)) => state.sim.residents.iter().find(|r| r.id == *id).map(|r| resident_card(state, r)).unwrap_or_else(|| "이 사람은 더 이상 없습니다".into()),
        Some(Selected::Group(ids)) => group_card(state, ids),
        Some(Selected::Village(id)) => state.sim.settlements.iter().find(|v| v.id == *id).map(|v| village_card(state, v)).unwrap_or_else(|| "이 마을은 흩어졌습니다".into()),
        Some(Selected::Animal(id)) => state.sim.animals.iter().find(|a| a.id == *id).map(|a|
            format!("동물\n몸무게 {:.0}kg\n\n체력   {}\n배고픔 {}\n\n겁 {:.0}% · 공격성 {:.0}%", a.archetype.body_mass_kg, bar(a.health), bar(a.hunger), a.archetype.fear * 100.0, a.archetype.aggression * 100.0)
        ).unwrap_or_else(|| "이 동물은 더 이상 없습니다".into()),
        Some(Selected::Monster(id)) => state.sim.monsters.iter().find(|m| m.id == *id).map(|m|
            format!("괴물\n몸무게 {:.0}kg\n\n체력   {}\n굶주림 {}\n\n공격성 {:.0}% · 갑옷 {:.0}% · 지능 {:.0}%", m.archetype.body_mass_kg, bar(m.health), bar(m.hunger), m.archetype.aggression * 100.0, m.archetype.armor * 100.0, m.archetype.intelligence * 100.0)
        ).unwrap_or_else(|| "이 괴물은 더 이상 없습니다".into()),
        Some(Selected::Project(id)) => state.sim.projects.iter().find(|p| p.id == *id).map(|p|
            format!("공사 중인 집\n\n작업   {}\n자재   {}\n\n크기 {:.1} × {:.1}m", bar(p.progress / p.required_work.max(0.1)), bar(p.material_committed / p.material_required.max(0.1)), p.design.length_m, p.design.width_m)
        ).unwrap_or_else(|| "공사가 끝났거나 사라졌습니다".into()),
        Some(Selected::Structure(id)) => state.sim.structures.iter().find(|s| s.id == *id).map(|s| {
            let cap = s.capabilities();
            format!("집\n{:.0}년에 완성 · 설계 {}세대\n\n튼튼함 {}\n\n쉼터 {}\n저장   {}\n작업   {}\n방어   {}",
                s.completed_year, s.design.generation, bar(s.integrity), bar(cap.shelter), bar(cap.storage), bar(cap.workspace), bar(cap.defense))
        }).unwrap_or_else(|| "이 건물은 무너졌습니다".into()),
        Some(Selected::Settlement) => {
            let alive = state.sim.residents.iter().filter(|r| r.health > 0.0).count();
            let villages: Vec<_> = state.sim.settlements.iter().filter(|x| !x.members.is_empty()).collect();
            let mut s = format!("세계 현황\n인구 {} · 가구 {} · 마을 {} · 건물 {}\n\n", alive, state.sim.households.len(), villages.len(), state.sim.structures.len());
            for v in villages.iter().take(8) {
                s.push_str(&format!("{} 마을 · {}명 · {}\n", names::village(&v.lexicon), v.members.len(), temperament(v)));
            }
            s
        }
    }
}
