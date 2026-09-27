use bevy::{prelude::*, window::PrimaryWindow};
use sim_core::{affordances::FeatureKind, awareness::SituationKind, events::WorldEventKind, sandbox::Sandbox, blueprints::{MonsterBlueprint,PixelCell,PixelSkin}, species::{AnimalArchetype,MonsterArchetype}, world::Position};

#[derive(Component)] struct WorldCamera;
#[derive(Component)] struct ResidentSprite(u64);
#[derive(Component)] struct MonsterSprite(u64);
#[derive(Component)] struct AnimalSprite(u64);
#[derive(Component)] struct FeatureSprite(u64);
#[derive(Component)] struct HudText;
#[derive(Component)] struct InspectorText;
#[derive(Component)] struct ToolText;
#[derive(Component)] struct EventOverlay(u64);
#[derive(Component)] struct ToolPreview;
#[derive(Component,Clone,Copy)] struct GodButton(GodTool);
#[derive(Component)] struct ToolContextText;
#[derive(Component)] struct MonsterLabText;
#[derive(Component)] struct MonsterLabPanel;
#[derive(Component,Clone,Copy)] struct PixelButton{ x:u8,y:u8 }
#[derive(Component)] struct MonsterLabGrid;

#[derive(Clone,Copy,Debug,PartialEq,Eq)] enum ToolCategory { Observe, Life, Nature, Disaster }
impl GodTool { fn category(self)->ToolCategory { match self {
    Self::Inspect=>ToolCategory::Observe,
    Self::Resident|Self::Animal|Self::Monster=>ToolCategory::Life,
    Self::Vegetation|Self::Mineral|Self::Rain|Self::Drought=>ToolCategory::Nature,
    Self::Fire|Self::Flood|Self::Earthquake=>ToolCategory::Disaster,
} } }

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
    animal:AnimalArchetype, monster:MonsterArchetype,
    monster_lab:bool, monster_blueprint:MonsterBlueprint, monster_mirror:bool,
}
impl Default for ViewerState {
    fn default()->Self { let seed=847_291; Self{sim:Sandbox::new(seed),seed,speed:1.0,paused:false,debug:true,selected:None,tool:GodTool::Inspect,tool_radius:70.0,tool_intensity:0.8,animal:AnimalArchetype::default(),monster:MonsterArchetype::default(),
            monster_lab:false,monster_blueprint:MonsterBlueprint{id:1,name:"Custom".into(),skin:PixelSkin::new(16,16),archetype:MonsterArchetype::default(),scale:1.0},monster_mirror:true} }
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
        .add_systems(Update,(controls,god_button_interactions,pixel_editor_interactions,tick_sim,sync_world,tool_preview,world_click,update_ui))
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
        Node{position_type:PositionType::Absolute,bottom:px(62),left:percent(20),..default()},ToolText));
    commands.spawn((Text::new(""),TextFont::from_font_size(13.0),TextColor(Color::srgb(0.84,0.86,0.78)),
        Node{position_type:PositionType::Absolute,bottom:px(64),right:px(14),..default()},ToolContextText));
    commands.spawn((Text::new(""),TextFont::from_font_size(12.0),TextColor(Color::srgb(0.92,0.92,0.86)),
        Node{position_type:PositionType::Absolute,top:px(100),left:px(14),..default()},Visibility::Hidden,MonsterLabText));
    commands.spawn((Node{
        position_type:PositionType::Absolute,top:px(92),left:px(18),width:px(360),height:px(430),
        display:Display::Grid,grid_template_columns:RepeatedGridTrack::flex(16,1.0),
        grid_template_rows:RepeatedGridTrack::flex(16,1.0),row_gap:px(1),column_gap:px(1),
        padding:UiRect::all(px(8)),..default()
    },BackgroundColor(Color::srgba(0.035,0.045,0.038,0.96)),Visibility::Hidden,MonsterLabPanel,MonsterLabGrid))
    .with_children(|p|{
        for y in 0..16u8 { for x in 0..16u8 {
            p.spawn((Button,Node{width:percent(100),height:percent(100),..default()},
                BackgroundColor(Color::srgb(0.09,0.10,0.09)),PixelButton{x,y}));
        }}
    });
    commands.spawn((Sprite::from_color(Color::srgba(0.95,0.90,0.65,0.10),Vec2::splat(140.0)),Transform::from_xyz(0.0,0.0,0.4),Visibility::Hidden,ToolPreview));
    commands.spawn((Node{
        position_type:PositionType::Absolute,bottom:px(10),left:percent(14),right:percent(14),height:px(46),
        display:Display::Flex,flex_direction:FlexDirection::Row,justify_content:JustifyContent::Center,
        align_items:AlignItems::Center,column_gap:px(5),padding:UiRect::all(px(5)),..default()
    },BackgroundColor(Color::srgba(0.035,0.045,0.038,0.88)))).with_children(|p|{
        for (tool,label) in [
            (GodTool::Inspect,"OBS"),(GodTool::Resident,"HUM"),(GodTool::Animal,"ANI"),(GodTool::Monster,"MON"),
            (GodTool::Vegetation,"VEG"),(GodTool::Mineral,"ORE"),(GodTool::Rain,"RAN"),(GodTool::Drought,"DRY"),
            (GodTool::Fire,"FIR"),(GodTool::Flood,"FLD"),(GodTool::Earthquake,"QUK")
        ] {
            p.spawn((Button,Node{width:px(44),height:px(34),justify_content:JustifyContent::Center,align_items:AlignItems::Center,..default()},
                BackgroundColor(Color::srgb(0.11,0.13,0.11)),GodButton(tool)))
             .with_child((Text::new(label),TextFont::from_font_size(11.0),TextColor(Color::srgb(0.90,0.88,0.78))));
        }
    });

}

fn god_button_interactions(
    mut q:Query<(&Interaction,&GodButton,&mut BackgroundColor),(Changed<Interaction>,With<Button>)>,
    mut state:ResMut<ViewerState>,
) {
    for (interaction,button,mut bg) in &mut q {
        match *interaction {
            Interaction::Pressed=>{state.tool=button.0;*bg=BackgroundColor(Color::srgb(0.30,0.28,0.16));}
            Interaction::Hovered=>{*bg=BackgroundColor(Color::srgb(0.20,0.21,0.16));}
            Interaction::None=>{*bg=BackgroundColor(Color::srgb(0.11,0.13,0.11));}
        }
    }
}

fn pixel_editor_interactions(
    mut q:Query<(&Interaction,&PixelButton,&mut BackgroundColor),(Changed<Interaction>,With<Button>)>,
    mut state:ResMut<ViewerState>,
) {
    if !state.monster_lab{return;}
    for (interaction,pixel,mut bg) in &mut q {
        if *interaction==Interaction::Pressed {
            let Some(i)=state.monster_blueprint.skin.index(pixel.x,pixel.y) else{continue;};
            let next=!state.monster_blueprint.skin.pixels[i].filled;
            state.monster_blueprint.skin.pixels[i]=PixelCell{filled:next,palette:1,emissive:false};
            if state.monster_mirror {
                let mx=state.monster_blueprint.skin.width-1-pixel.x;
                if let Some(mi)=state.monster_blueprint.skin.index(mx,pixel.y) {
                    state.monster_blueprint.skin.pixels[mi]=PixelCell{filled:next,palette:1,emissive:false};
                }
            }
            *bg=BackgroundColor(if next{Color::srgb(0.72,0.22,0.16)}else{Color::srgb(0.09,0.10,0.09)});
        }
    }
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
    if keys.just_pressed(KeyCode::KeyJ){state.monster.body_mass_kg=(state.monster.body_mass_kg*0.75).max(10.0);}
    if keys.just_pressed(KeyCode::KeyK){state.monster.body_mass_kg=(state.monster.body_mass_kg*1.33).min(20_000.0);}
    if keys.just_pressed(KeyCode::KeyU){state.monster.speed=(state.monster.speed-0.1).max(0.05);}
    if keys.just_pressed(KeyCode::KeyY){state.monster.speed=(state.monster.speed+0.1).min(2.0);}
    if keys.just_pressed(KeyCode::KeyB){state.monster.aggression=(state.monster.aggression-0.1).max(0.0);}
    if keys.just_pressed(KeyCode::KeyP){state.monster.aggression=(state.monster.aggression+0.1).min(1.0);}
    if keys.just_pressed(KeyCode::KeyL){state.monster_lab=!state.monster_lab;state.monster_blueprint.archetype=state.monster;}
    if keys.just_pressed(KeyCode::KeySemicolon){state.monster_mirror=!state.monster_mirror;}
    if state.monster_lab {
        if keys.just_pressed(KeyCode::KeyC){for p in &mut state.monster_blueprint.skin.pixels{*p=PixelCell::default();}}
        if keys.just_pressed(KeyCode::KeyR){for y in 0..state.monster_blueprint.skin.height{for x in 0..state.monster_blueprint.skin.width{let on=((x as u64*17+y as u64*31+state.seed)%7)<3;if on{state.monster_blueprint.skin.set(x,y,PixelCell{filled:true,palette:1,emissive:false});}}}}
    }
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
    mut features:Query<(Entity,&FeatureSprite,&mut Transform)>,
) {
    for (tag,mut t,mut sprite) in &mut residents {
        if let Some(r)=state.sim.residents.iter().find(|r|r.id==tag.0) {
            t.translation.x=r.position.x;t.translation.y=r.position.y;
            sprite.color=if r.health<=0.0 {Color::srgb(0.20,0.16,0.14)} else {Color::srgb(0.88,0.76,0.48)};
        }
    }
    let existing_features:Vec<u64>=features.iter().map(|(_,f,_)|f.0).collect();
    for f in &state.sim.features {
        if !existing_features.contains(&f.id) {
            let (color,size,z)=match f.kind {
                FeatureKind::Vegetation=>(Color::srgb(0.12,0.40,0.16),Vec2::new(7.0,10.0),-1.0),
                FeatureKind::RockFace=>(Color::srgb(0.36,0.36,0.32),Vec2::new(11.0,11.0),-1.0),
                FeatureKind::DeepWater=>(Color::srgba(0.10,0.34,0.42,0.35),Vec2::new(38.0,20.0),-2.0),
                FeatureKind::LooseMaterial=>(Color::srgb(0.56,0.48,0.34),Vec2::new(6.0,6.0),-0.8),
                _=>(Color::srgb(0.45,0.42,0.30),Vec2::splat(6.0),-1.0),
            };
            commands.spawn((Sprite::from_color(color,size),Transform::from_xyz(f.position.x,f.position.y,z),FeatureSprite(f.id)));
        }
    }
    for (_,tag,mut t) in &mut features {
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

fn tool_preview(
    window:Query<&Window,With<PrimaryWindow>>,
    camera:Query<(&Camera,&GlobalTransform),With<WorldCamera>>,
    state:Res<ViewerState>,
    mut preview:Query<(&mut Transform,&mut Sprite,&mut Visibility),With<ToolPreview>>,
) {
    let Ok(w)=window.single() else{return;}; let Some(cursor)=w.cursor_position() else{return;};
    let Ok((cam,global))=camera.single() else{return;}; let Ok(world)=cam.viewport_to_world_2d(global,cursor) else{return;};
    let Ok((mut t,mut sprite,mut vis))=preview.single_mut() else{return;};
    if state.tool==GodTool::Inspect { *vis=Visibility::Hidden; return; }
    *vis=Visibility::Visible; t.translation.x=world.x;t.translation.y=world.y;
    let radius=match state.tool {GodTool::Resident|GodTool::Animal|GodTool::Monster=>12.0,GodTool::Vegetation|GodTool::Mineral=>32.0,_=>state.tool_radius};
    sprite.custom_size=Some(Vec2::splat(radius*2.0));
    sprite.color=match state.tool {
        GodTool::Fire=>Color::srgba(0.95,0.25,0.08,0.13),
        GodTool::Flood|GodTool::Rain=>Color::srgba(0.18,0.50,0.82,0.12),
        GodTool::Drought=>Color::srgba(0.78,0.62,0.25,0.12),
        GodTool::Earthquake=>Color::srgba(0.75,0.62,0.35,0.12),
        GodTool::Monster=>Color::srgba(0.85,0.15,0.12,0.16),
        GodTool::Resident=>Color::srgba(0.90,0.78,0.48,0.15),
        GodTool::Animal=>Color::srgba(0.70,0.62,0.42,0.15),
        GodTool::Vegetation=>Color::srgba(0.15,0.55,0.18,0.13),
        GodTool::Mineral=>Color::srgba(0.55,0.48,0.35,0.15),
        _=>Color::srgba(0.95,0.90,0.65,0.10),
    };
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
        GodTool::Animal=>state.sim.spawn_animal_with(p,state.animal),
        GodTool::Monster=>state.sim.spawn_monster_with(p,state.monster),
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
    mut context:Query<&mut Text,(With<ToolContextText>,Without<ToolText>,Without<HudText>,Without<InspectorText>)>,
    mut lab:Query<(&mut Text,&mut Visibility),(With<MonsterLabText>,Without<ToolContextText>,Without<ToolText>,Without<HudText>,Without<InspectorText>)>,
) {
    if let Ok(mut t)=hud.single_mut() {
        let alive=state.sim.residents.iter().filter(|r|r.health>0.0).count();
        t.0=format!("SEED {}   YEAR {:.2}   RESIDENTS {}/{}   MONSTERS {}\nSPEED x{} {}   [1-5] speed [Space] pause [R] seed [F3] debug\n[WASD] pan [+/-] zoom [V] settlement pulse",
            state.seed,state.sim.year,alive,state.sim.residents.len(),state.sim.monsters.iter().filter(|m|m.health>0.0).count(),
            state.speed as u32,if state.paused{"PAUSED"}else{""});
    }
    if let Ok(mut t)=tool.single_mut(){t.0=format!("GOD DOCK  [I] Inspect [H] Human [Z] Animal [M] Monster [T] Trees [O] Ore [N] Rain [X] Drought [F] Fire [G] Flood [Q] Quake\nACTIVE: {}   radius {:.0}   intensity {:.1}   [[ / ]] radius   [, / .] power",state.tool.label(),state.tool_radius,state.tool_intensity);}
    if let Ok(mut t)=context.single_mut(){
        let cat=match state.tool.category(){ToolCategory::Observe=>"OBSERVE",ToolCategory::Life=>"LIFE",ToolCategory::Nature=>"NATURE",ToolCategory::Disaster=>"DISASTER"};
        let hint=match state.tool {GodTool::Inspect=>"click an entity",GodTool::Resident=>"spawn autonomous resident",GodTool::Animal=>"spawn wildlife",GodTool::Monster=>"spawn hostile pressure",GodTool::Vegetation=>"grow local vegetation",GodTool::Mineral=>"deposit material patch",GodTool::Rain=>"local rainfall event",GodTool::Drought=>"local water stress",GodTool::Fire=>"local fire pressure",GodTool::Flood=>"local flood pressure",GodTool::Earthquake=>"local seismic pressure"};
        t.0=if state.tool==GodTool::Monster {
            format!("{} / {}\n{}\nMASS {:.0}kg [J/K]  SPEED {:.2} [U/Y]\nAGGR {:.2} [B/P]  armor {:.2}  intel {:.2}",cat,state.tool.label(),hint,state.monster.body_mass_kg,state.monster.speed,state.monster.aggression,state.monster.armor,state.monster.intelligence)
        } else if state.tool==GodTool::Animal {
            format!("{} / {}\n{}\nMASS {:.0}kg  SPEED {:.2}\nFEAR {:.2}  AGGR {:.2}",cat,state.tool.label(),hint,state.animal.body_mass_kg,state.animal.speed,state.animal.fear,state.animal.aggression)
        } else {format!("{} / {}\n{}\nradius {:.0} · power {:.1}",cat,state.tool.label(),hint,state.tool_radius,state.tool_intensity)};
    }
    if let Ok((mut t,mut vis))=lab.single_mut(){
        *vis=if state.monster_lab{Visibility::Visible}else{Visibility::Hidden};
        if state.monster_lab {
            let skin=&state.monster_blueprint.skin;let mut grid=String::new();
            for y in 0..skin.height{for x in 0..skin.width{let p=skin.pixels[skin.index(x,y).unwrap()];grid.push(if p.filled{'#'}else{'·'});}grid.push('\n');}
            t.0=format!("MONSTER LAB — {}x{}  [L close] [C clear] [R seed silhouette]\n{}\ncoverage {:.0}% · scale {:.1}\nMASS {:.0}kg SPEED {:.2} AGGR {:.2}\n(pixel mouse editor next)",skin.width,skin.height,grid,skin.filled_fraction()*100.0,state.monster_blueprint.scale,state.monster.body_mass_kg,state.monster.speed,state.monster.aggression);
        }
    }
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
                format!("MONSTER #{}\nHP {:.0}%  hunger {:.2}\nMASS {:.0}kg speed {:.2}\naggr {:.2} armor {:.2} intel {:.2}\nposition {:.0}, {:.0}",m.id,m.health*100.0,m.hunger,m.archetype.body_mass_kg,m.archetype.speed,m.archetype.aggression,m.archetype.armor,m.archetype.intelligence,m.position.x,m.position.y)
            ).unwrap_or_else(||"monster no longer exists".into()),
        };
    }
}
