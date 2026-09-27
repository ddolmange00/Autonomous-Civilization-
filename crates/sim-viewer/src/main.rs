use bevy::{prelude::*, window::PrimaryWindow};
use sim_core::{affordances::FeatureKind, awareness::SituationKind, events::WorldEventKind, sandbox::Sandbox, world::Position};

#[derive(Component)] struct WorldCamera;
#[derive(Component)] struct ResidentSprite(u64);
#[derive(Component)] struct MonsterSprite(u64);
#[derive(Component)] struct AnimalSprite(u64);
#[derive(Component)] struct FeatureSprite(u64);
#[derive(Component)] struct HudText;
#[derive(Component)] struct InspectorText;
#[derive(Component)] struct ToolText;
#[derive(Component)] struct EventOverlay(u64);

#[derive(Clone, Copy, Debug)]
enum Selected { Resident(u64), Monster(u64), Settlement }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum GodTool { Inspect, Resident, Animal, Monster, Vegetation, Mineral, Rain, Drought, Fire, Flood, Earthquake }
impl GodTool { fn label(self)->&'static str { match self {
    Self::Inspect=>"INSPECT",Self::Resident=>"RESIDENT",Self::Animal=>"ANIMAL",Self::Monster=>"MONSTER",
    Self::Vegetation=>"VEGETATION",Self::Mineral=>"MINERAL",Self::Rain=>"RAIN",Self::Drought=>"DROUGHT",
    Self::Fire=>"FIRE",Self::Flood=>"FLOOD",Self::Earthquake=>"QUAKE"
} } }

#[derive(Resource)]
struct ViewerState {
    sim:Sandbox, seed:u64, speed:f32, paused:bool, debug:bool, selected:Option<Selected>,
    tool:GodTool, tool_radius:f32, tool_intensity:f32,
}
impl Default for ViewerState {
    fn default()->Self { let seed=847_291; Self{sim:Sandbox::new(seed),seed,speed:1.0,paused:false,debug:true,selected:None,tool:GodTool::Inspect,tool_radius:70.0,tool_intensity:0.8} }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.055,0.075,0.060)))
        .init_resource::<ViewerState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            primary_window:Some(Window{title:"Autonomous Civilization — Sim Viewer".into(),..default()}),
            ..default()
        }))
        .add_systems(Startup,setup)
        .add_systems(Update,(controls,tick_sim,sync_world,world_click,update_ui))
        .run();
}

fn setup(mut commands:Commands,state:Res<ViewerState>) {
    commands.spawn((Camera2d,WorldCamera));
    commands.spawn((Sprite::from_color(Color::srgb(0.20,0.29,0.20),Vec2::new(900.0,600.0)),Transform::from_xyz(0.0,0.0,-5.0)));
    commands.spawn((Sprite::from_color(Color::srgb(0.08,0.25,0.31),Vec2::new(46.0,560.0)),Transform::from_xyz(0.0,0.0,-3.0)));
    for f in &state.sim.features {
        let (color,size,z)=match f.kind {
            FeatureKind::Vegetation=>(Color::srgb(0.12,0.40,0.16),Vec2::new(7.0,10.0),-1.0),
            FeatureKind::RockFace=>(Color::srgb(0.36,0.36,0.32),Vec2::new(11.0,11.0),-1.0),
            FeatureKind::DeepWater=>(Color::srgba(0.10,0.34,0.42,0.35),Vec2::new(38.0,20.0),-2.0),
            _=>(Color::srgb(0.45,0.42,0.30),Vec2::splat(6.0),-1.0),
        };
        commands.spawn((Sprite::from_color(color,size),Transform::from_xyz(f.position.x,f.position.y,z),FeatureSprite(f.id)));
    }
    for r in &state.sim.residents {
        commands.spawn((Sprite::from_color(Color::srgb(0.88,0.76,0.48),Vec2::new(7.0,10.0)),
            Transform::from_xyz(r.position.x,r.position.y,1.0),ResidentSprite(r.id)));
    }
    commands.spawn((Text::new(""),TextFont::from_font_size(15.0),TextColor(Color::WHITE),
        Node{position_type:PositionType::Absolute,top:px(10),left:px(12),..default()},HudText));
    commands.spawn((Text::new(""),TextFont::from_font_size(14.0),TextColor(Color::srgb(0.88,0.92,0.86)),
        Node{position_type:PositionType::Absolute,top:px(10),right:px(12),..default()},InspectorText));
    commands.spawn((Text::new(""),TextFont::from_font_size(15.0),TextColor(Color::srgb(0.96,0.90,0.72)),
        Node{position_type:PositionType::Absolute,bottom:px(14),left:percent(20),..default()},ToolText));
}

fn controls(
    keys:Res<ButtonInput<KeyCode>>,time:Res<Time>,mut state:ResMut<ViewerState>,
    mut camera:Query<&mut Transform,With<WorldCamera>>,mut commands:Commands,
    monsters:Query<Entity,With<MonsterSprite>>,
) {
    if keys.just_pressed(KeyCode::Digit1){state.speed=1.0;}
    if keys.just_pressed(KeyCode::Digit2){state.speed=5.0;}
    if keys.just_pressed(KeyCode::Digit3){state.speed=20.0;}
    if keys.just_pressed(KeyCode::Digit4){state.speed=100.0;}
    if keys.just_pressed(KeyCode::Digit5){state.speed=1000.0;}
    if keys.just_pressed(KeyCode::Space){state.paused=!state.paused;}
    if keys.just_pressed(KeyCode::F3){state.debug=!state.debug;}
    if keys.just_pressed(KeyCode::KeyI){state.tool=GodTool::Inspect;}
    if keys.just_pressed(KeyCode::KeyH){state.tool=GodTool::Resident;}
    if keys.just_pressed(KeyCode::KeyZ){state.tool=GodTool::Animal;}
    if keys.just_pressed(KeyCode::KeyT){state.tool=GodTool::Vegetation;}
    if keys.just_pressed(KeyCode::KeyO){state.tool=GodTool::Mineral;}
    if keys.just_pressed(KeyCode::KeyN){state.tool=GodTool::Rain;}
    if keys.just_pressed(KeyCode::KeyX){state.tool=GodTool::Drought;}
    if keys.just_pressed(KeyCode::KeyM){state.tool=GodTool::Monster;}
    if keys.just_pressed(KeyCode::KeyF){state.tool=GodTool::Fire;}
    if keys.just_pressed(KeyCode::KeyG){state.tool=GodTool::Flood;}
    if keys.just_pressed(KeyCode::KeyQ){state.tool=GodTool::Earthquake;}
    if keys.just_pressed(KeyCode::BracketLeft){state.tool_radius=(state.tool_radius-10.0).max(10.0);}
    if keys.just_pressed(KeyCode::BracketRight){state.tool_radius=(state.tool_radius+10.0).min(240.0);}
    if keys.just_pressed(KeyCode::Comma){state.tool_intensity=(state.tool_intensity-0.1).max(0.1);}
    if keys.just_pressed(KeyCode::Period){state.tool_intensity=(state.tool_intensity+0.1).min(2.0);}
    if keys.just_pressed(KeyCode::KeyV){state.selected=Some(Selected::Settlement);}
    if keys.just_pressed(KeyCode::KeyR){
        state.seed=state.seed.wrapping_add(1); state.sim=Sandbox::new(state.seed); state.selected=None;
        for e in &monsters { commands.entity(e).despawn(); }
    }
    if let Ok(mut t)=camera.single_mut() {
        let mut d=Vec2::ZERO;
        if keys.pressed(KeyCode::KeyA)||keys.pressed(KeyCode::ArrowLeft){d.x-=1.0;}
        if keys.pressed(KeyCode::KeyD)||keys.pressed(KeyCode::ArrowRight){d.x+=1.0;}
        if keys.pressed(KeyCode::KeyW)||keys.pressed(KeyCode::ArrowUp){d.y+=1.0;}
        if keys.pressed(KeyCode::KeyS)||keys.pressed(KeyCode::ArrowDown){d.y-=1.0;}
        if d.length_squared()>0.0 { let scale=t.scale.x; t.translation+=(d.normalize()*260.0*time.delta_secs()*scale).extend(0.0); }
        if keys.pressed(KeyCode::Equal){t.scale*=1.0-time.delta_secs()*0.8;}
        if keys.pressed(KeyCode::Minus){t.scale*=1.0+time.delta_secs()*0.8;}
        t.scale.x=t.scale.x.clamp(0.35,4.0); t.scale.y=t.scale.x;
    }
}

fn tick_sim(time:Res<Time>,mut state:ResMut<ViewerState>) {
    if state.paused{return;}
    let days=time.delta_secs()*0.55*state.speed;
    let steps=(days/2.0).ceil().clamp(1.0,120.0) as usize;
    let dt=days/steps as f32;
    for _ in 0..steps { state.sim.step(dt); }
}

fn sync_world(
    mut commands:Commands,state:Res<ViewerState>,
    mut residents:Query<(&ResidentSprite,&mut Transform,&mut Sprite)>,
    mut monsters:Query<(Entity,&MonsterSprite,&mut Transform,&mut Sprite)>,
    mut animals:Query<(Entity,&AnimalSprite,&mut Transform,&mut Sprite)>,
    mut features:Query<(&FeatureSprite,&mut Transform)>,
) {
    for (tag,mut t,mut sprite) in &mut residents {
        if let Some(r)=state.sim.residents.iter().find(|r|r.id==tag.0) {
            t.translation.x=r.position.x;t.translation.y=r.position.y;
            sprite.color=if r.health<=0.0 {Color::srgb(0.20,0.16,0.14)} else {Color::srgb(0.88,0.76,0.48)};
        }
    }
    for (tag,mut t) in &mut features {
        if let Some(f)=state.sim.features.iter().find(|f|f.id==tag.0){t.translation.x=f.position.x;t.translation.y=f.position.y;}
    }
    let existing_animals:Vec<u64>=animals.iter().map(|(_,a,_,_)|a.0).collect();
    for a in &state.sim.animals {
        if !existing_animals.contains(&a.id) {
            commands.spawn((Sprite::from_color(Color::srgb(0.70,0.62,0.42),Vec2::new(9.0,7.0)),
                Transform::from_xyz(a.position.x,a.position.y,1.1),AnimalSprite(a.id)));
        }
    }
    for (e,tag,mut t,mut sprite) in &mut animals {
        if let Some(a)=state.sim.animals.iter().find(|a|a.id==tag.0) {
            t.translation.x=a.position.x;t.translation.y=a.position.y;
            sprite.color=if a.health<=0.0 {Color::srgb(0.20,0.16,0.12)} else {Color::srgb(0.70,0.62,0.42)};
        } else {commands.entity(e).despawn();}
    }
    let existing:Vec<u64>=monsters.iter().map(|(_,m,_,_)|m.0).collect();
    for m in &state.sim.monsters {
        if !existing.contains(&m.id) {
            commands.spawn((Sprite::from_color(Color::srgb(0.72,0.16,0.13),Vec2::splat(15.0)),
                Transform::from_xyz(m.position.x,m.position.y,1.2),MonsterSprite(m.id)));
        }
    }
    for (e,tag,mut t,mut sprite) in &mut monsters {
        if let Some(m)=state.sim.monsters.iter().find(|m|m.id==tag.0) {
            t.translation.x=m.position.x;t.translation.y=m.position.y;
            sprite.color=if m.health<=0.0 {Color::srgb(0.20,0.08,0.07)} else {Color::srgb(0.72,0.16,0.13)};
        } else { commands.entity(e).despawn(); }
    }
}

fn world_click(
    buttons:Res<ButtonInput<MouseButton>>,window:Query<&Window,With<PrimaryWindow>>,
    camera:Query<(&Camera,&GlobalTransform,&Transform),With<WorldCamera>>,mut state:ResMut<ViewerState>,
    mut commands:Commands,
) {
    if !buttons.just_pressed(MouseButton::Left){return;}
    let Ok(w)=window.single() else{return;}; let Some(cursor)=w.cursor_position() else{return;};
    let Ok((cam,global,cam_t))=camera.single() else{return;};
    let Ok(world)=cam.viewport_to_world_2d(global,cursor) else{return;};
    let p=Position{x:world.x,y:world.y};
    match state.tool {
        GodTool::Inspect=>{
            let threshold=18.0*cam_t.scale.x; let mut best:(f32,Option<Selected>)=(threshold,None);
            for r in &state.sim.residents {let d=world.distance(Vec2::new(r.position.x,r.position.y));if d<best.0{best=(d,Some(Selected::Resident(r.id)));}}
            for m in &state.sim.monsters {let d=world.distance(Vec2::new(m.position.x,m.position.y));if d<best.0{best=(d,Some(Selected::Monster(m.id)));}}
            state.selected=best.1;
        }
        GodTool::Resident=>state.sim.spawn_resident_at(p),
        GodTool::Animal=>state.sim.spawn_animal_at(p),
        GodTool::Monster=>state.sim.spawn_monster_at(p),
        GodTool::Vegetation=>state.sim.grow_vegetation_at(p,(8.0+state.tool_intensity*12.0) as u32),
        GodTool::Mineral=>state.sim.deposit_minerals_at(p,(4.0+state.tool_intensity*7.0) as u32),
        GodTool::Rain|GodTool::Drought|GodTool::Fire|GodTool::Flood|GodTool::Earthquake=>{
            let (kind,duration,color)=match state.tool {
                GodTool::Rain=>(WorldEventKind::Rain,18.0,Color::srgba(0.30,0.55,0.85,0.12)),
                GodTool::Drought=>(WorldEventKind::Drought,90.0,Color::srgba(0.78,0.62,0.25,0.13)),
                GodTool::Fire=>(WorldEventKind::Fire,24.0,Color::srgba(0.95,0.25,0.08,0.20)),
                GodTool::Flood=>(WorldEventKind::Flood,18.0,Color::srgba(0.12,0.48,0.78,0.18)),
                _=>(WorldEventKind::Earthquake,2.0,Color::srgba(0.75,0.62,0.35,0.16)),
            };
            let radius=state.tool_radius; let intensity=state.tool_intensity;
            let id=state.sim.inject_event(kind,p,radius,intensity,duration);
            commands.spawn((Sprite::from_color(color,Vec2::splat(radius*2.0)),Transform::from_xyz(p.x,p.y,0.5),EventOverlay(id)));
        }
    }
}

fn update_ui(
    state:Res<ViewerState>,mut hud:Query<&mut Text,(With<HudText>,Without<InspectorText>)>,
    mut inspector:Query<&mut Text,(With<InspectorText>,Without<HudText>)>,
    mut tool:Query<&mut Text,(With<ToolText>,Without<HudText>,Without<InspectorText>)>,
) {
    if let Ok(mut t)=hud.single_mut() {
        let alive=state.sim.residents.iter().filter(|r|r.health>0.0).count();
        t.0=format!("SEED {}   YEAR {:.2}   RESIDENTS {}/{}   MONSTERS {}\nSPEED x{} {}   [1-5] speed [Space] pause [R] seed [F3] debug\n[WASD] pan [+/-] zoom [V] settlement pulse",
            state.seed,state.sim.year,alive,state.sim.residents.len(),state.sim.monsters.iter().filter(|m|m.health>0.0).count(),
            state.speed as u32,if state.paused{"PAUSED"}else{""});
    }
    if let Ok(mut t)=tool.single_mut(){t.0=format!("GOD DOCK  [I] Inspect [H] Human [Z] Animal [M] Monster [T] Trees [O] Ore [N] Rain [X] Drought [F] Fire [G] Flood [Q] Quake\nACTIVE: {}   radius {:.0}   intensity {:.1}   [[ / ]] radius   [, / .] power",state.tool.label(),state.tool_radius,state.tool_intensity);}
    if let Ok(mut t)=inspector.single_mut() {
        t.0=match state.selected {
            None=>"CLICK AN ENTITY\n\nF3 toggles internal cognition".into(),
            Some(Selected::Resident(id))=>state.sim.residents.iter().find(|r|r.id==id).map(|r|{
                let mut s=format!("RESIDENT #{}\nHP {:.0}%   ACTION {:?}\n\nNeeds\nhunger {:.2} safety {:.2} curiosity {:.2}\n",r.id,r.health*100.0,r.current_action,r.mind.needs.hunger,r.mind.needs.safety,r.mind.needs.curiosity);
                if state.debug {
                    s.push_str(&format!("\nTraits\naggr {:.2} risk {:.2} curious {:.2}\nempathy {:.2} conform {:.2} persist {:.2}\n\nDecision scores\n",
                        r.mind.traits.aggression,r.mind.traits.risk_tolerance,r.mind.traits.curiosity,r.mind.traits.empathy,r.mind.traits.conformity,r.mind.traits.persistence));
                    for q in &r.top_scores {s.push_str(&format!("{:?}: {:+.3}\n",q.action,q.score));}
                    s.push_str(&format!("\nMemory episodes {}",r.memory.episodes.len()));
                }
                s
            }).unwrap_or_else(||"resident no longer exists".into()),
            Some(Selected::Monster(id))=>state.sim.monsters.iter().find(|m|m.id==id).map(|m|
                format!("MONSTER #{}\nHP {:.0}%\nhunger {:.2}\naggression {:.2}\nposition {:.0}, {:.0}",m.id,m.health*100.0,m.hunger,m.aggression,m.position.x,m.position.y)
            ).unwrap_or_else(||"monster no longer exists".into()),
        };
    }
}
