#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Response { Fight, Fortify, Migrate, Hide, Scatter }

#[derive(Clone, Copy, Debug)]
pub struct ThreatContext {
    pub threat: f32,
    pub defense: f32,
    pub food_days: f32,
    pub mobility: f32,
    pub shelter: f32,
    pub risk_tolerance: f32,
}

pub fn choose_response(c: ThreatContext) -> Response {
    let ratio=c.defense/c.threat.max(.001);
    if ratio>1.15 && c.risk_tolerance>.35 { return Response::Fight; }
    if c.shelter>.7 && ratio>.55 { return Response::Fortify; }
    if c.mobility>.55 && c.food_days>8.0 { return Response::Migrate; }
    if c.shelter>.45 { return Response::Hide; }
    Response::Scatter
}
