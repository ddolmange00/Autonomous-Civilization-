mod capture;
mod cards;
mod effects;
mod names;
mod select;
mod sprites;
mod terrain_view;
mod ui;

use bevy::{prelude::*, window::PrimaryWindow};
use sim_core::terrain::{palette::MapMode, MapSize, TileMap, WorldTemplate};
use sim_core::{affordances::FeatureKind, events::WorldEventKind, sandbox::{Preset,Sandbox}, blueprint_library::BlueprintLibrary, blueprints::{AnchorKind,MonsterBlueprint,PixelCell,PixelSkin}, pixel_animation::{anchored_pixel_offset,body_transform,pixel_offset}, species::{AnimalArchetype,MonsterArchetype}, world::Position};

#[derive(Component)] pub struct WorldCamera;
/// A sprite that mirrors one simulation entity.
#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)] enum ActorKind { Resident, Feature, Animal, Project, Structure }
#[derive(Component)] struct Actor{kind:ActorKind,id:u64}
/// Walk-cycle state for residents: last drawn position and cycle phase.
#[derive(Component)] struct Walker{last:Vec2,phase:f32,moving:f32,facing_left:bool}
#[derive(Component)] struct MonsterSprite(u64);
/// Animated flame standing on one burning tile.
#[derive(Component)] struct Flame(u32);
#[derive(Component)] struct MonsterPixel{ monster_id:u64,x:u8,y:u8,width:u8,height:u8,base_x:f32,base_y:f32 }
#[derive(Component)] pub(crate) struct InspectorText;
#[derive(Component)] struct EventOverlay(u64);
#[derive(Component)] struct ToolPreview;
#[derive(Component)] pub(crate) struct WorldDynamic;
#[derive(Component)] pub(crate) struct ToolContextText;
#[derive(Component)] struct MonsterLabText;
#[derive(Component)] struct MonsterLabPanel;
#[derive(Component,Clone,Copy)] struct PixelButton{ x:u8,y:u8 }
#[derive(Component)] struct MonsterLabGrid;
#[derive(Resource,Default)] struct LabUiState{built_side:u8,painting:bool,erase:bool}

#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub(crate) enum ToolCategory { Observe, Life, Nature, Disaster }
impl GodTool { fn category(self)->ToolCategory { match self {
    Self::Inspect=>ToolCategory::Observe,
    Self::Resident|Self::Animal|Self::Monster=>ToolCategory::Life,
    Self::Vegetation|Self::Mineral|Self::Rock|Self::Water|Self::Rain|Self::Drought=>ToolCategory::Nature,
    Self::Fire|Self::Flood|Self::Earthquake=>ToolCategory::Disaster,
} } }

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Selected { Resident(u64), Animal(u64), Monster(u64), Project(u64), Structure(u64), Settlement, Village(u64), Group(Vec<u64>) }

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum GodTool { Inspect, Resident, Animal, Monster, Vegetation, Mineral, Rock, Water, Rain, Drought, Fire, Flood, Earthquake }


#[derive(Resource)]
pub struct ViewerState {
    pub sim:Sandbox, seed:u64,
    pub template:WorldTemplate, pub map_size:MapSize, pub map_mode:MapMode, pub world_generation:u64,
    cursor_tile:String, recenter:bool, speed:f32, paused:bool, debug:bool, selected:Option<Selected>,
    pub tool:GodTool, tool_radius:f32, tool_intensity:f32,
    animal:AnimalArchetype, monster:MonsterArchetype,
    pub monster_lab:bool, monster_blueprint:MonsterBlueprint, monster_mirror:bool,
    monster_palette:u8, monster_emissive:bool, monster_fill:bool, blueprint_library:BlueprintLibrary,
    anchor_mode:Option<AnchorKind>,
}
impl Default for ViewerState {
    fn default()->Self { let seed=847_291; let (template,map_size)=(capture::template_from_env().unwrap_or(WorldTemplate::Continents),MapSize::Large);
        Self{sim:build_world(seed,map_size,template),seed,template,map_size,map_mode:MapMode::Terrain,world_generation:0,cursor_tile:String::new(),recenter:true,speed:1.0,paused:false,debug:false,selected:None,tool:GodTool::Inspect,tool_radius:70.0,tool_intensity:0.8,animal:AnimalArchetype::default(),monster:MonsterArchetype::default(),
            monster_lab:false,monster_blueprint:MonsterBlueprint{id:1,name:"Custom".into(),skin:PixelSkin::new(16,16),archetype:MonsterArchetype::default(),scale:1.0,anchors:vec![]},monster_mirror:true,monster_palette:1,monster_emissive:false,monster_fill:false,blueprint_library:BlueprintLibrary{monsters:vec![],capacity:32},anchor_mode:None} }
}

fn build_world(seed:u64,size:MapSize,template:WorldTemplate)->Sandbox {
    Sandbox::on_terrain(seed,Preset::Default,TileMap::generate(seed,size,template))
}

impl ViewerState {
    pub fn rebuild_world(&mut self) {
        self.sim=build_world(self.seed,self.map_size,self.template);
        self.world_generation+=1;
        self.selected=None;
        self.recenter=true;
    }
}

fn main() {
    let mut app=App::new();
    app
        .insert_resource(ClearColor(Color::srgb(0.055,0.075,0.060)))
        .init_resource::<ViewerState>()
        .init_resource::<LabUiState>()
        .init_resource::<sprites::SpriteBank>()
        .init_resource::<ui::UiState>()
        .init_resource::<effects::Shake>()
        .init_resource::<effects::FxRng>()
        .init_resource::<select::DragSelect>()
        .add_plugins(DefaultPlugins.set(WindowPlugin{
            primary_window:Some(Window{title:"Autonomous Civilization — Sim Viewer".into(),..default()}),
            ..default()
        }))
        .add_systems(Startup,(ui::install_font,setup,ui::spawn_ui,select::spawn_box))
        .add_systems(Update,(ui::ui_buttons,ui::ui_layout,ui::ui_status,ui::sync_banners,select::box_select))
        .add_systems(Update,(effects::spawn_effects,effects::update_particles,effects::camera_shake))
        .add_systems(Update,(controls,rebuild_monster_grid,pixel_editor_interactions,sync_monster_lab,tick_sim,sync_world,tool_preview,world_click,update_ui))
        .add_systems(Update,(terrain_view::sync_terrain,terrain_view::map_camera,recenter_camera,sync_flames));
    capture::install(&mut app);
    app.run();
}

fn setup(mut commands:Commands,mut bank:ResMut<sprites::SpriteBank>,mut images:ResMut<Assets<Image>>) {
    let (disc,_)=bank.get(sprites::SpriteKey::Disc,&mut images);
    commands.spawn((Camera2d,WorldCamera));
    commands.spawn((Text::new(""),TextFont::from_font_size(12.0),TextColor(Color::srgb(0.92,0.92,0.86)),
        Node{position_type:PositionType::Absolute,top:px(100),left:px(14),..default()},Visibility::Hidden,MonsterLabText));
    commands.spawn((Node{
        position_type:PositionType::Absolute,top:px(92),left:px(18),width:px(430),height:px(430),
        display:Display::Grid,row_gap:px(1),column_gap:px(1),padding:UiRect::all(px(8)),..default()
    },BackgroundColor(Color::srgba(0.035,0.045,0.038,0.96)),Visibility::Hidden,MonsterLabPanel,MonsterLabGrid));
    commands.spawn((Sprite{image:disc,color:Color::srgba(0.95,0.90,0.65,0.10),custom_size:Some(Vec2::splat(140.0)),..default()},Transform::from_xyz(0.0,0.0,60.0),Visibility::Hidden,ToolPreview));
}

fn rebuild_monster_grid(
    mut commands:Commands,state:Res<ViewerState>,mut ui:ResMut<LabUiState>,
    mut grid:Query<(Entity,&mut Node),With<MonsterLabGrid>>,
) {
    if !state.monster_lab{return;}
    let side=state.monster_blueprint.skin.width;
    if ui.built_side==side{return;}
    let Ok((entity,mut node))=grid.single_mut() else{return;};
    commands.entity(entity).despawn_related::<Children>();
    node.grid_template_columns=RepeatedGridTrack::flex(side as u16,1.0);
    node.grid_template_rows=RepeatedGridTrack::flex(side as u16,1.0);
    commands.entity(entity).with_children(|p|{
        for y in 0..side {for x in 0..side {
            p.spawn((Button,Node{width:percent(100),height:percent(100),..default()},
                BackgroundColor(Color::srgb(0.09,0.10,0.09)),PixelButton{x,y}));
        }}
    });
    ui.built_side=side;
}

fn pixel_editor_interactions(
    buttons:Res<ButtonInput<MouseButton>>,
    mut q:Query<(&Interaction,&PixelButton,&mut BackgroundColor),With<Button>>,
    mut state:ResMut<ViewerState>,mut ui:ResMut<LabUiState>,
) {
    if !state.monster_lab{return;}
    if buttons.just_pressed(MouseButton::Left){ui.painting=true;ui.erase=false;}
    if buttons.just_pressed(MouseButton::Right){ui.painting=true;ui.erase=true;}
    if buttons.just_released(MouseButton::Left)||buttons.just_released(MouseButton::Right){ui.painting=false;}
    for (interaction,pixel,mut bg) in &mut q {
        if *interaction!=Interaction::Hovered && *interaction!=Interaction::Pressed {continue;}
        if !ui.painting && *interaction!=Interaction::Pressed {continue;}
        if state.monster_fill && *interaction==Interaction::Pressed {
            let cell=PixelCell{filled:!ui.erase,palette:state.monster_palette,emissive:state.monster_emissive};
            state.monster_blueprint.skin.flood_fill(pixel.x,pixel.y,cell);continue;
        }
        let cell=PixelCell{filled:!ui.erase,palette:state.monster_palette,emissive:state.monster_emissive};
        if let Some(i)=state.monster_blueprint.skin.index(pixel.x,pixel.y){state.monster_blueprint.skin.pixels[i]=cell;}
        if state.monster_mirror {
            let mx=state.monster_blueprint.skin.width-1-pixel.x;
            if let Some(mi)=state.monster_blueprint.skin.index(mx,pixel.y){state.monster_blueprint.skin.pixels[mi]=cell;}
        }
        *bg=BackgroundColor(if cell.filled{Color::srgb(0.72,0.22,0.16)}else{Color::srgb(0.09,0.10,0.09)});
    }
}

fn sync_monster_lab(
    state:Res<ViewerState>,
    mut panel:Query<&mut Visibility,With<MonsterLabPanel>>,
    mut pixels:Query<(&PixelButton,&mut BackgroundColor)>,
) {
    if let Ok(mut v)=panel.single_mut(){*v=if state.monster_lab{Visibility::Visible}else{Visibility::Hidden};}
    if state.monster_lab {
        for (p,mut bg) in &mut pixels {
            if let Some(i)=state.monster_blueprint.skin.index(p.x,p.y) {
                let cell=state.monster_blueprint.skin.pixels[i];
                let normal=match cell.palette{2=>Color::srgb(0.18,0.48,0.72),3=>Color::srgb(0.48,0.68,0.28),4=>Color::srgb(0.62,0.42,0.72),_=>Color::srgb(0.72,0.22,0.16)};
                *bg=BackgroundColor(if cell.filled{if cell.emissive{Color::srgb(0.95,0.72,0.18)}else{normal}}else{Color::srgb(0.09,0.10,0.09)});
            }
        }
    }
}

fn controls(
    keys:Res<ButtonInput<KeyCode>>,time:Res<Time>,mut state:ResMut<ViewerState>,
    mut camera:Query<&mut Transform,With<WorldCamera>>,mut commands:Commands,
    dynamic:Query<Entity,With<WorldDynamic>>,
) {
    if keys.just_pressed(KeyCode::Digit1){state.speed=1.0;}
    if keys.just_pressed(KeyCode::Digit2){state.speed=5.0;}
    if keys.just_pressed(KeyCode::Digit3){state.speed=20.0;}
    if keys.just_pressed(KeyCode::Digit4){state.speed=100.0;}
    if keys.just_pressed(KeyCode::Digit5){state.speed=1000.0;}
    if keys.just_pressed(KeyCode::Space){state.paused=!state.paused;}
    if keys.just_pressed(KeyCode::F3)&&!state.monster_lab{state.debug=!state.debug;}
    if keys.just_pressed(KeyCode::KeyI){state.tool=GodTool::Inspect;}
    if keys.just_pressed(KeyCode::KeyH){state.tool=GodTool::Resident;}
    if keys.just_pressed(KeyCode::KeyZ){state.tool=GodTool::Animal;}
    if keys.just_pressed(KeyCode::KeyT){state.tool=GodTool::Vegetation;}
    if keys.just_pressed(KeyCode::KeyO){state.tool=GodTool::Mineral;}
    if keys.just_pressed(KeyCode::KeyN){state.tool=GodTool::Rain;}
    if keys.just_pressed(KeyCode::KeyX){state.tool=GodTool::Drought;}
    if keys.just_pressed(KeyCode::KeyM){state.tool=GodTool::Monster;}
    if keys.just_pressed(KeyCode::KeyF)&&!state.monster_lab{state.tool=GodTool::Fire;}
    if keys.just_pressed(KeyCode::KeyG){state.tool=GodTool::Flood;}
    if keys.just_pressed(KeyCode::KeyQ){state.tool=GodTool::Earthquake;}
    if keys.just_pressed(KeyCode::BracketLeft){state.tool_radius=(state.tool_radius-10.0).max(10.0);}
    if keys.just_pressed(KeyCode::BracketRight){state.tool_radius=(state.tool_radius+10.0).min(240.0);}
    if keys.just_pressed(KeyCode::Comma){state.tool_intensity=(state.tool_intensity-0.1).max(0.1);}
    if keys.just_pressed(KeyCode::Period){state.tool_intensity=(state.tool_intensity+0.1).min(2.0);}
    if keys.just_pressed(KeyCode::KeyJ){state.monster.body_mass_kg=(state.monster.body_mass_kg*0.75).max(10.0);}
    if keys.just_pressed(KeyCode::KeyK){state.monster.body_mass_kg=(state.monster.body_mass_kg*1.33).min(20_000.0);}
    if keys.just_pressed(KeyCode::KeyU){state.monster.speed=(state.monster.speed-0.1).max(0.05);}
    if keys.just_pressed(KeyCode::KeyY){state.monster.speed=(state.monster.speed+0.1).min(2.0);}
    if keys.just_pressed(KeyCode::KeyB){state.monster.aggression=(state.monster.aggression-0.1).max(0.0);}
    if keys.just_pressed(KeyCode::KeyP){state.monster.aggression=(state.monster.aggression+0.1).min(1.0);}
    if keys.just_pressed(KeyCode::KeyL){state.monster_lab=!state.monster_lab;state.monster_blueprint.archetype=state.monster;}
    if keys.just_pressed(KeyCode::Semicolon){state.monster_mirror=!state.monster_mirror;}
    if keys.just_pressed(KeyCode::KeyE){state.monster_emissive=!state.monster_emissive;}
    if keys.just_pressed(KeyCode::F1)&&state.monster_lab{state.anchor_mode=Some(AnchorKind::Head);}
    if keys.just_pressed(KeyCode::F2)&&state.monster_lab{state.anchor_mode=Some(AnchorKind::Eye);}
    if keys.just_pressed(KeyCode::F3)&&state.monster_lab{state.anchor_mode=Some(AnchorKind::Foot);}
    if keys.just_pressed(KeyCode::F4)&&state.monster_lab{state.anchor_mode=Some(AnchorKind::Tail);}
    if keys.just_pressed(KeyCode::F5)&&state.monster_lab{state.anchor_mode=Some(AnchorKind::Attack);}
    if keys.just_pressed(KeyCode::Escape)&&state.monster_lab{state.anchor_mode=None;}
    if keys.just_pressed(KeyCode::KeyF)&&state.monster_lab{state.monster_fill=!state.monster_fill;}
    if keys.just_pressed(KeyCode::Digit6)&&state.monster_lab{state.monster_palette=1;}
    if keys.just_pressed(KeyCode::Digit7)&&state.monster_lab{state.monster_palette=2;}
    if keys.just_pressed(KeyCode::Digit8)&&state.monster_lab{state.monster_palette=3;}
    if keys.just_pressed(KeyCode::Digit9)&&state.monster_lab{state.monster_palette=4;}
    if keys.just_pressed(KeyCode::F6)&&state.monster_lab{state.monster_blueprint.skin=state.monster_blueprint.skin.resize_nearest(16,16);}
    if keys.just_pressed(KeyCode::F7)&&state.monster_lab{state.monster_blueprint.skin=state.monster_blueprint.skin.resize_nearest(24,24);}
    if keys.just_pressed(KeyCode::F8)&&state.monster_lab{state.monster_blueprint.skin=state.monster_blueprint.skin.resize_nearest(32,32);}
    if keys.just_pressed(KeyCode::F9)&&state.monster_lab{let arch=state.monster;state.monster_blueprint.archetype=arch;let bp=state.monster_blueprint.clone();state.blueprint_library.save(bp);}
    if keys.just_pressed(KeyCode::F10)&&state.monster_lab&&!state.blueprint_library.monsters.is_empty(){let next=state.blueprint_library.monsters.iter().position(|b|b.id==state.monster_blueprint.id).map(|i|(i+1)%state.blueprint_library.monsters.len()).unwrap_or(0);state.monster_blueprint=state.blueprint_library.monsters[next].clone();state.monster=state.monster_blueprint.archetype;}
    if keys.just_pressed(KeyCode::F11)&&state.monster_lab&&!state.blueprint_library.monsters.is_empty(){let prev=state.blueprint_library.monsters.iter().position(|b|b.id==state.monster_blueprint.id).map(|i|(i+state.blueprint_library.monsters.len()-1)%state.blueprint_library.monsters.len()).unwrap_or(0);state.monster_blueprint=state.blueprint_library.monsters[prev].clone();state.monster=state.monster_blueprint.archetype;}
    if state.monster_lab {
        if keys.just_pressed(KeyCode::KeyC){for p in &mut state.monster_blueprint.skin.pixels{*p=PixelCell::default();}}
        if keys.just_pressed(KeyCode::KeyR){for y in 0..state.monster_blueprint.skin.height{for x in 0..state.monster_blueprint.skin.width{let on=((x as u64*17+y as u64*31+state.seed)%7)<3;if on{state.monster_blueprint.skin.set(x,y,PixelCell{filled:true,palette:1,emissive:false});}}}}
    }
    if keys.just_pressed(KeyCode::KeyV){state.selected=Some(Selected::Settlement);}
    let mut rebuild=false;
    if keys.just_pressed(KeyCode::KeyR)&&!state.monster_lab{ state.seed=state.seed.wrapping_add(1); rebuild=true; }
    if keys.just_pressed(KeyCode::Backquote)&&!state.monster_lab{
        let i=WorldTemplate::ALL.iter().position(|t|*t==state.template).unwrap_or(0);
        state.template=WorldTemplate::ALL[(i+1)%WorldTemplate::ALL.len()]; rebuild=true;
    }
    if keys.just_pressed(KeyCode::Backslash)&&!state.monster_lab{
        let i=MapSize::ALL.iter().position(|t|*t==state.map_size).unwrap_or(0);
        state.map_size=MapSize::ALL[(i+1)%MapSize::ALL.len()]; rebuild=true;
    }
    if keys.just_pressed(KeyCode::Tab){ state.map_mode=state.map_mode.next(); }
    if rebuild {
        state.rebuild_world();
        for e in &dynamic { commands.entity(e).despawn(); }
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
    }
}

fn tick_sim(time:Res<Time>,mut state:ResMut<ViewerState>) {
    if state.paused{return;}
    let days=time.delta_secs()*0.55*state.speed;
    let steps=(days/2.0).ceil().clamp(1.0,120.0) as usize;
    let dt=days/steps as f32;
    for _ in 0..steps { state.sim.step(dt); }
}

/// What one simulation entity should look like this frame.
struct Wanted{pos:Position,key:sprites::SpriteKey,size_mul:f32,tint:Color,dead:bool,shadow:f32,action:Option<sim_core::agency::ActionPrimitive>}

fn wanted_actors(state:&ViewerState)->std::collections::HashMap<(ActorKind,u64),Wanted> {
    use sprites::{Look,SpriteKey,TreeKind,steps4};
    use sim_core::life_history::{LifeStage,Sex};
    let t=state.sim.terrain.as_deref();
    let mut out=std::collections::HashMap::new();
    for r in &state.sim.residents {
        let family=r.life.kinship.household.unwrap_or(r.id);
        let look=Look::new(r.life.biological.pigmentation,family,r.id,r.life.sex==Sex::Female);
        let size_mul=match r.life.stage(state.sim.year){LifeStage::Infant=>0.45,LifeStage::Child=>0.62,LifeStage::Adolescent=>0.82,_=>1.0};
        let dead=r.health<=0.0;
        out.insert((ActorKind::Resident,r.id),Wanted{pos:r.position,key:SpriteKey::Person(look,0),size_mul,
            tint:if dead{Color::srgb(0.55,0.5,0.5)}else{Color::WHITE},dead,shadow:8.0*size_mul,action:Some(r.current_action)});
    }
    for f in &state.sim.features {
        let q=(f.quantity/f.capacity.max(0.001)).clamp(0.0,1.0);
        let key=match f.kind {
            FeatureKind::Vegetation=>{
                let biome=t.and_then(|m|m.tile_at(f.position).map(|(x,y)|m.biome[m.index(x,y)]));
                SpriteKey::Tree(biome.map(TreeKind::for_biome).unwrap_or(TreeKind::Broadleaf),steps4(q),(f.id%3) as u8)
            }
            FeatureKind::RockFace=>SpriteKey::Boulder((f.id%3) as u8),
            FeatureKind::LooseMaterial=>SpriteKey::Stones(steps4(q)),
            _=>continue,
        };
        let shadow=match key{SpriteKey::Tree(..)=>11.0,SpriteKey::Boulder(_)=>14.0,_=>0.0};
        out.insert((ActorKind::Feature,f.id),Wanted{pos:f.position,key,size_mul:1.0,tint:Color::WHITE,dead:false,shadow,action:None});
    }
    for a in &state.sim.animals {
        out.insert((ActorKind::Animal,a.id),Wanted{pos:a.position,key:SpriteKey::Animal((a.id%4) as u8,a.health<=0.0),size_mul:1.0,tint:Color::WHITE,dead:false,shadow:10.0,action:None});
    }
    for p in &state.sim.projects {
        let ratio=(p.progress/p.required_work.max(0.1)).clamp(0.0,1.0);
        out.insert((ActorKind::Project,p.id),Wanted{pos:p.position,key:SpriteKey::Site(steps4(ratio)),size_mul:1.0,tint:Color::WHITE,dead:false,shadow:0.0,action:None});
    }
    for s in &state.sim.structures {
        let material=if s.material_invested<20.0{0}else if s.material_invested<45.0{1}else{2};
        let integrity=(s.integrity.clamp(0.0,1.0)*3.0).round() as u8;
        out.insert((ActorKind::Structure,s.id),Wanted{pos:s.position,key:SpriteKey::House(material,integrity),size_mul:1.0,tint:Color::WHITE,dead:false,shadow:40.0,action:None});
    }
    out
}

fn event_colour(kind:WorldEventKind)->Color {
    match kind {
        WorldEventKind::Rain=>Color::srgba(0.30,0.55,0.85,0.18),
        WorldEventKind::Drought=>Color::srgba(0.78,0.62,0.25,0.20),
        WorldEventKind::Fire=>Color::srgba(0.95,0.30,0.08,0.30),
        WorldEventKind::Flood=>Color::srgba(0.12,0.48,0.78,0.26),
        _=>Color::srgba(0.75,0.62,0.35,0.22),
    }
}

fn sync_world(
    mut commands:Commands,state:Res<ViewerState>,time:Res<Time>,
    mut images:ResMut<Assets<Image>>,mut bank:ResMut<sprites::SpriteBank>,
    mut actors:Query<(Entity,&Actor,&mut Transform,&mut Sprite,Option<&mut Walker>),(Without<MonsterSprite>,Without<MonsterPixel>)>,
    mut monsters:Query<(Entity,&MonsterSprite,&mut Transform,Option<&mut Sprite>),(Without<Actor>,Without<MonsterPixel>)>,
    mut monster_pixels:Query<(&MonsterPixel,&mut Transform),(Without<Actor>,Without<MonsterSprite>)>,
    overlays:Query<(Entity,&EventOverlay)>,
) {
    use std::collections::{HashMap,HashSet};
    let t=state.sim.terrain.as_deref();
    let mut wanted=wanted_actors(&state);
    let (disc,_)=bank.get(sprites::SpriteKey::Disc,&mut images);
    for (e,actor,mut tf,mut sprite,walker) in &mut actors {
        let Some(w)=wanted.remove(&(actor.kind,actor.id)) else{commands.entity(e).despawn();continue;};
        let v=terrain_view::to_view(t,w.pos);
        let mut key=w.key;
        let mut bob=0.0;
        if let (Some(mut walk),sprites::SpriteKey::Person(look,_))=(walker,key) {
            // Walk while moving (facing the way they go), swing a tool while working,
            // breathe while idle. Moving is remembered briefly so slow steps still animate.
            let dt=if state.paused{0.0}else{time.delta_secs()};
            let delta=v-walk.last;
            if delta.length()>0.02 { walk.moving=0.3; if delta.x.abs()>0.01 {walk.facing_left=delta.x<0.0;} } else { walk.moving-=dt; }
            walk.last=v;
            walk.phase+=dt;
            let seed=(actor.id%7) as f32*0.37;
            let pose=if w.dead {sprites::POSE_STAND}
                else if walk.moving>0.0 {((walk.phase*8.0) as u32%4) as u8}
                else if w.action.map(cards::is_work).unwrap_or(false) {if ((walk.phase*3.0+seed) as u32)%2==0{sprites::POSE_WORK_UP}else{sprites::POSE_WORK_DOWN}}
                else { if ((walk.phase*1.3+seed) % 1.0) < 0.5 {bob=1.0;} sprites::POSE_STAND };
            key=sprites::SpriteKey::Person(look,pose);
            sprite.flip_x=walk.facing_left;
        }
        let (image,size)=bank.get(key,&mut images);
        if sprite.image!=image {sprite.image=image;}
        sprite.custom_size=Some(size*w.size_mul);
        sprite.color=w.tint;
        tf.translation=Vec3::new(v.x,v.y+bob,terrain_view::depth(t,w.pos));
        tf.rotation=if w.dead{Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)}else{Quat::IDENTITY};
    }
    for ((kind,id),w) in wanted {
        let v=terrain_view::to_view(t,w.pos);
        let (image,size)=bank.get(w.key,&mut images);
        let mut entity=commands.spawn((
            Sprite{image,color:w.tint,custom_size:Some(size*w.size_mul),..default()},
            bevy::sprite::Anchor::BOTTOM_CENTER,
            Transform::from_xyz(v.x,v.y,terrain_view::depth(t,w.pos)),
            Actor{kind,id},WorldDynamic,
        ));
        if kind==ActorKind::Resident {entity.insert(Walker{last:v,phase:0.0,moving:0.0,facing_left:false});}
        if w.shadow>0.0 {
            entity.with_child((Sprite{image:disc.clone(),color:Color::srgba(0.0,0.0,0.0,0.28),custom_size:Some(Vec2::new(w.shadow,w.shadow*0.45)),..default()},
                Transform::from_xyz(0.0,0.0,-0.02)));
        }
    }

    let existing:HashSet<u64>=monsters.iter().map(|(_,m,_,_)|m.0).collect();
    for m in &state.sim.monsters {
        if existing.contains(&m.id){continue;}
        let v=terrain_view::to_view(t,m.position);
        let at=Transform::from_xyz(v.x,v.y,terrain_view::depth(t,m.position));
        if let Some(skin)=&m.skin {
            let scale=(18.0/skin.width.max(skin.height) as f32).max(0.5);
            commands.spawn((at,MonsterSprite(m.id),WorldDynamic))
                .with_children(|p|{
                    for y in 0..skin.height {for x in 0..skin.width {
                        let cell=skin.pixels[skin.index(x,y).unwrap()]; if !cell.filled{continue;}
                        let px=(x as f32-(skin.width as f32-1.0)*0.5)*scale;
                        let py=((skin.height as f32-1.0)-y as f32)*scale;
                        p.spawn((Sprite::from_color(if cell.emissive{Color::srgb(0.95,0.72,0.18)}else{Color::srgb(0.72,0.16,0.13)},Vec2::splat(scale)),
                            Transform::from_xyz(px,py,0.0),MonsterPixel{monster_id:m.id,x,y,width:skin.width,height:skin.height,base_x:px,base_y:py}));
                    }}
                });
        } else {
            commands.spawn((Sprite::from_color(Color::srgb(0.72,0.16,0.13),Vec2::new(14.0,16.0)),bevy::sprite::Anchor::BOTTOM_CENTER,at,MonsterSprite(m.id),WorldDynamic));
        }
    }
    let monsters_by_id:HashMap<u64,usize>=state.sim.monsters.iter().enumerate().map(|(i,m)|(m.id,i)).collect();
    for (e,tag,mut tf,sprite) in &mut monsters {
        let Some(m)=monsters_by_id.get(&tag.0).map(|&i|&state.sim.monsters[i]) else{commands.entity(e).despawn();continue;};
        let v=terrain_view::to_view(t,m.position);
        let bt=body_transform(m.motion,m.motion_phase,m.archetype.speed);
        tf.translation=Vec3::new(v.x+bt.offset_x,v.y+bt.offset_y,terrain_view::depth(t,m.position));
        tf.scale.x=bt.scale_x;tf.scale.y=bt.scale_y;tf.rotation=Quat::from_rotation_z(bt.rotation);
        if let Some(mut sprite)=sprite {sprite.color=if m.health<=0.0 {Color::srgb(0.20,0.08,0.07)} else {Color::srgb(0.72,0.16,0.13)};}
    }
    for (tag,mut tf) in &mut monster_pixels {
        if let Some(m)=monsters_by_id.get(&tag.monster_id).map(|&i|&state.sim.monsters[i]) {
            let (dx,dy)=if let Some(bp)=&m.blueprint{anchored_pixel_offset(bp,tag.x,tag.y,m.motion,m.motion_phase)}else{pixel_offset(tag.x,tag.y,tag.width,tag.height,m.motion,m.motion_phase)};
            tf.translation.x=tag.base_x+dx;tf.translation.y=tag.base_y+dy;
        }
    }

    // Active events are drawn as ground ellipses; finished ones disappear.
    let mut live:HashMap<u64,&sim_core::events::WorldEvent>=state.sim.events.iter().map(|e|(e.id,e)).collect();
    for (e,overlay) in &overlays {
        if live.remove(&overlay.0).is_none() {commands.entity(e).despawn();}
    }
    for (id,ev) in live {
        // Tile fields already show fire and floodwater where they really are.
        if t.is_some()&&matches!(ev.kind,WorldEventKind::Fire|WorldEventKind::Flood) {continue;}
        let v=terrain_view::to_view(t,ev.position);
        commands.spawn((Sprite{image:disc.clone(),color:event_colour(ev.kind),custom_size:Some(terrain_view::ground_ellipse(t,ev.radius)),..default()},
            Transform::from_xyz(v.x,v.y,0.5),EventOverlay(id),WorldDynamic));
    }
}

fn tool_preview(
    window:Query<&Window,With<PrimaryWindow>>,
    camera:Query<(&Camera,&GlobalTransform),With<WorldCamera>>,
    mut state:ResMut<ViewerState>,
    mut preview:Query<(&mut Transform,&mut Sprite,&mut Visibility),With<ToolPreview>>,
) {
    let Ok(w)=window.single() else{return;}; let Some(cursor)=w.cursor_position() else{return;};
    let Ok((cam,global))=camera.single() else{return;}; let Ok(view)=cam.viewport_to_world_2d(global,cursor) else{return;};
    let label=state.sim.terrain.as_deref().and_then(|t|terrain_view::cursor_tile_label(t,view)).unwrap_or_default();
    if state.cursor_tile!=label {state.cursor_tile=label;}
    let Ok((mut t,mut sprite,mut vis))=preview.single_mut() else{return;};
    if state.tool==GodTool::Inspect { *vis=Visibility::Hidden; return; }
    // Snap the preview to the ground point under the cursor.
    let ground=terrain_view::from_view(state.sim.terrain.as_deref(),view);
    let at=terrain_view::to_view(state.sim.terrain.as_deref(),ground);
    *vis=Visibility::Visible; t.translation.x=at.x;t.translation.y=at.y;
    let radius=match state.tool {GodTool::Resident|GodTool::Animal|GodTool::Monster=>6.0,GodTool::Vegetation|GodTool::Mineral=>32.0,GodTool::Rock|GodTool::Water=>state.tool_radius*0.35,_=>state.tool_radius};
    sprite.custom_size=Some(terrain_view::ground_ellipse(state.sim.terrain.as_deref(),radius));
    sprite.color=match state.tool {
        GodTool::Fire=>Color::srgba(0.95,0.25,0.08,0.25),
        GodTool::Flood|GodTool::Rain=>Color::srgba(0.18,0.50,0.82,0.25),
        GodTool::Drought=>Color::srgba(0.78,0.62,0.25,0.25),
        GodTool::Earthquake=>Color::srgba(0.75,0.62,0.35,0.25),
        GodTool::Monster=>Color::srgba(0.85,0.15,0.12,0.30),
        GodTool::Resident=>Color::srgba(0.90,0.78,0.48,0.30),
        GodTool::Animal=>Color::srgba(0.70,0.62,0.42,0.30),
        GodTool::Vegetation=>Color::srgba(0.15,0.55,0.18,0.25),
        GodTool::Mineral=>Color::srgba(0.55,0.48,0.35,0.28),
        GodTool::Rock=>Color::srgba(0.45,0.45,0.40,0.30),
        GodTool::Water=>Color::srgba(0.10,0.34,0.55,0.32),
        _=>Color::srgba(0.95,0.90,0.65,0.20),
    };
}

fn world_click(
    buttons:Res<ButtonInput<MouseButton>>,window:Query<&Window,With<PrimaryWindow>>,
    camera:Query<(&Camera,&GlobalTransform,&Transform),With<WorldCamera>>,mut state:ResMut<ViewerState>,
    ui_buttons:Query<&Interaction>,
) {
    if !buttons.just_pressed(MouseButton::Left){return;}
    if state.monster_lab || ui_buttons.iter().any(|i|*i!=Interaction::None){return;}
    let Ok(w)=window.single() else{return;}; let Some(cursor)=w.cursor_position() else{return;};
    let Ok((cam,global,cam_t))=camera.single() else{return;};
    let Ok(view)=cam.viewport_to_world_2d(global,cursor) else{return;};
    let terrain=state.sim.terrain.clone();
    let t=terrain.as_deref();
    let p=terrain_view::from_view(t,view);
    let tool=state.tool;
    let animal=state.animal;
    let monster=state.monster;
    let skin=state.monster_blueprint.skin.clone();
    let blueprint=state.monster_blueprint.clone();
    let intensity=state.tool_intensity;
    let radius=state.tool_radius;
    match tool {
        GodTool::Inspect=>{
            // Pick in view space against the sprite body (a little above the feet).
            let threshold=14.0*cam_t.scale.x.max(0.6); let mut best:(f32,Option<Selected>)=(threshold,None);
            let d=|pos:Position,lift:f32|view.distance(terrain_view::to_view(t,pos)+Vec2::new(0.0,lift));
            for r in &state.sim.residents {let d=d(r.position,8.0);if d<best.0{best=(d,Some(Selected::Resident(r.id)));}}
            for a in &state.sim.animals {let d=d(a.position,4.0);if d<best.0{best=(d,Some(Selected::Animal(a.id)));}}
            for p in &state.sim.projects {let d=d(p.position,8.0);if d<best.0{best=(d,Some(Selected::Project(p.id)));}}
            for s in &state.sim.structures {let d=d(s.position,10.0);if d<best.0{best=(d,Some(Selected::Structure(s.id)));}}
            for m in &state.sim.monsters {let d=d(m.position,9.0);if d<best.0{best=(d,Some(Selected::Monster(m.id)));}}
            state.selected=best.1;
        }
        GodTool::Resident=>state.sim.spawn_resident_at(p),
        GodTool::Animal=>state.sim.spawn_animal_with(p,animal),
        GodTool::Monster=>state.sim.spawn_monster_with(p,monster,Some(skin),Some(blueprint)),
        GodTool::Vegetation=>state.sim.grow_vegetation_at(p,(8.0+intensity*12.0) as u32),
        GodTool::Mineral=>state.sim.deposit_minerals_at(p,(4.0+intensity*7.0) as u32),
        GodTool::Rock=>state.sim.raise_rock_at(p,radius),
        GodTool::Water=>state.sim.dig_water_at(p,radius),
        GodTool::Rain|GodTool::Drought|GodTool::Fire|GodTool::Flood|GodTool::Earthquake=>{
            let (kind,duration)=match tool {
                GodTool::Rain=>(WorldEventKind::Rain,18.0),
                GodTool::Drought=>(WorldEventKind::Drought,90.0),
                GodTool::Fire=>(WorldEventKind::Fire,24.0),
                GodTool::Flood=>(WorldEventKind::Flood,18.0),
                _=>(WorldEventKind::Earthquake,2.0),
            };
            // On a tile world fire, floodwater and rain act through the tile fields;
            // the event remains so residents can perceive it and tell stories about it.
            match tool {
                GodTool::Fire=>{state.sim.ignite_at(p,radius*0.35,intensity.min(1.0));}
                GodTool::Flood=>state.sim.flood_at(p,radius*0.35,intensity),
                GodTool::Rain=>state.sim.rain_at(p,radius,intensity),
                _=>{}
            }
            state.sim.inject_event(kind,p,radius,intensity,duration);
        }
    }
}

fn update_ui(
    state:Res<ViewerState>,time:Res<Time>,mut since:Local<f32>,mut shown:Local<Option<Selected>>,
    mut inspector:Query<&mut Text,(With<InspectorText>,Without<ToolContextText>,Without<MonsterLabText>)>,
    mut context:Query<&mut Text,(With<ToolContextText>,Without<InspectorText>,Without<MonsterLabText>)>,
    mut lab:Query<(&mut Text,&mut Visibility),(With<MonsterLabText>,Without<ToolContextText>,Without<InspectorText>)>,
) {
    if let Ok(mut t)=context.single_mut(){
        let cat=match state.tool.category(){ToolCategory::Observe=>"관찰",ToolCategory::Life=>"생명",ToolCategory::Nature=>"자연",ToolCategory::Disaster=>"재해"};
        let (name,hint)=match state.tool {
            GodTool::Inspect=>("관찰","대상을 클릭하면 정보가 보입니다"),GodTool::Resident=>("사람","스스로 판단하는 주민을 놓습니다"),
            GodTool::Animal=>("동물","야생동물을 풀어놓습니다"),GodTool::Monster=>("괴물","위협이 되는 괴물을 놓습니다"),
            GodTool::Vegetation=>("숲","식생을 자라게 합니다"),GodTool::Mineral=>("광석","광물 더미를 묻어둡니다"),
            GodTool::Rock=>("산","땅을 바위산으로 솟게 합니다"),GodTool::Water=>("물","땅을 파서 물웅덩이를 만듭니다"),
            GodTool::Rain=>("비","비를 내려 불을 끄고 땅을 적십니다"),GodTool::Drought=>("가뭄","물이 말라 식생이 줄어듭니다"),
            GodTool::Fire=>("불","불을 붙입니다. 연료와 바람을 따라 번집니다"),GodTool::Flood=>("홍수","물을 쏟습니다. 낮은 곳으로 흘러 고입니다"),
            GodTool::Earthquake=>("지진","땅을 흔들어 건물을 무너뜨립니다"),
        };
        t.0=if state.tool==GodTool::Monster {
            format!("{} / {}\n{}\n무게 {:.0}kg [J/K]  속도 {:.2} [U/Y]\n공격성 {:.2} [B/P]  갑옷 {:.2}  지능 {:.2}",cat,name,hint,state.monster.body_mass_kg,state.monster.speed,state.monster.aggression,state.monster.armor,state.monster.intelligence)
        } else if state.tool==GodTool::Animal {
            format!("{} / {}\n{}\n무게 {:.0}kg  속도 {:.2}\n겁 {:.2}  공격성 {:.2}",cat,name,hint,state.animal.body_mass_kg,state.animal.speed,state.animal.fear,state.animal.aggression)
        } else {format!("{} / {}\n{}\n반경 {:.0} [ ]   세기 {:.1} , .",cat,name,hint,state.tool_radius,state.tool_intensity)};
    }
    if let Ok((mut t,mut vis))=lab.single_mut(){
        *vis=if state.monster_lab{Visibility::Visible}else{Visibility::Hidden};
        if state.monster_lab {
            let skin=&state.monster_blueprint.skin;let mut grid=String::new();
            for y in 0..skin.height{for x in 0..skin.width{let p=skin.pixels[skin.index(x,y).unwrap()];grid.push(if p.filled{'#'}else{'.'});}grid.push('\n');}
            t.0=format!("MONSTER LAB — {}x{}  [L close] [C clear] [R seed silhouette] [; mirror {}]\n{}\ncoverage {:.0}% | scale {:.1}\nMASS {:.0}kg SPEED {:.2} AGGR {:.2}\nANCHOR {:?} | F1 head F2 eye F3 foot F4 tail F5 attack Esc paint\nLMB paint | RMB erase | F fill | E emissive\nF6/F7/F8 16/24/32 | F9 save | F10/F11 library",skin.width,skin.height,if state.monster_mirror{"ON"}else{"OFF"},grid,skin.filled_fraction()*100.0,state.monster_blueprint.scale,state.monster.body_mass_kg,state.monster.speed,state.monster.aggression,state.anchor_mode);
        }
    }
    // The card changes with the world but at a readable pace, not every tick.
    *since+=time.delta_secs();
    if let Ok(mut t)=inspector.single_mut() {
        if *since>=0.35 || *shown!=state.selected {
            *since=0.0;
            *shown=state.selected.clone();
            t.0=cards::inspector_text(&state);
        }
    }
}

/// After a new world is built, frame the founding band.
fn recenter_camera(mut state:ResMut<ViewerState>,mut camera:Query<&mut Transform,With<WorldCamera>>) {
    if !state.recenter {return;}
    let Ok(mut t)=camera.single_mut() else{return;};
    let alive:Vec<_>=state.sim.residents.iter().filter(|r|r.health>0.0).map(|r|Vec2::new(r.position.x,r.position.y)).collect();
    if !alive.is_empty() {
        let c=alive.iter().copied().sum::<Vec2>()/alive.len() as f32;
        let v=terrain_view::to_view(state.sim.terrain.as_deref(),Position{x:c.x,y:c.y});
        t.translation.x=v.x; t.translation.y=v.y;
    }
    t.scale=Vec3::new(0.5,0.5,1.0);
    state.recenter=false;
}

/// Burning tiles each carry a flickering flame; flames go out with the fire.
fn sync_flames(
    mut commands:Commands,state:Res<ViewerState>,time:Res<Time>,
    mut images:ResMut<Assets<Image>>,mut bank:ResMut<sprites::SpriteBank>,
    mut flames:Query<(Entity,&Flame,&mut Sprite,&mut Transform),Without<Actor>>,
) {
    const MAX_FLAMES:usize=3000;
    let (Some(t),Some(hz))=(state.sim.terrain.as_deref(),state.sim.hazards.as_ref()) else {
        for (e,..) in &flames {commands.entity(e).despawn();}
        return;
    };
    let wanted:std::collections::HashSet<u32>=hz.burning().iter().copied().take(MAX_FLAMES).collect();
    let mut shown=std::collections::HashSet::new();
    let clock=time.elapsed_secs()*9.0;
    for (e,flame,mut sprite,mut tf) in &mut flames {
        if !wanted.contains(&flame.0) {commands.entity(e).despawn();continue;}
        shown.insert(flame.0);
        let i=flame.0 as usize;
        let frame=((clock+(i%7) as f32) as u32%3) as u8;
        let (img,size)=bank.get(sprites::SpriteKey::Flame(frame),&mut images);
        if sprite.image!=img {sprite.image=img;}
        let k=0.6+hz.fire[i]*0.7;
        sprite.custom_size=Some(size*k);
        tf.translation.z=terrain_view::depth(Some(t),t.tile_center(i%t.width,i/t.width))+0.001;
    }
    for &i in wanted.difference(&shown) {
        let idx=i as usize;
        let p=t.tile_center(idx%t.width,idx/t.width);
        let v=terrain_view::to_view(Some(t),p);
        let (img,size)=bank.get(sprites::SpriteKey::Flame(0),&mut images);
        commands.spawn((Sprite{image:img,custom_size:Some(size),..default()},bevy::sprite::Anchor::BOTTOM_CENTER,
            Transform::from_xyz(v.x,v.y,terrain_view::depth(Some(t),p)+0.001),Flame(i),WorldDynamic));
    }
}
