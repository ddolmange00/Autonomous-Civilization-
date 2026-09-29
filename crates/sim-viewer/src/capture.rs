//! Headless-ish visual check: `SIM_VIEWER_CAPTURE=<dir>` frames the village,
//! the whole map, the biome map mode and a rebuilt world, saves a PNG of each, then exits.
//! Optional `SIM_VIEWER_TEMPLATE=<label>` picks the world template.
use bevy::{
    input::{mouse::{MouseButtonInput, MouseMotion, MouseScrollUnit, MouseWheel}, touch::TouchPhase, ButtonState},
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
    window::PrimaryWindow,
};
use sim_core::terrain::{palette::MapMode, WorldTemplate};

use crate::{ViewerState, WorldCamera};

#[derive(Resource)]
pub struct CaptureScript { dir: std::path::PathBuf, frame: u32 }

pub fn install(app: &mut App) {
    let Ok(dir) = std::env::var("SIM_VIEWER_CAPTURE") else { return; };
    app.insert_resource(CaptureScript { dir: dir.into(), frame: 0 })
        .insert_resource(InputProbe::default())
        .add_systems(First, inject_mouse)
        .add_systems(Update, run_capture);
}

pub fn template_from_env() -> Option<WorldTemplate> {
    let want = std::env::var("SIM_VIEWER_TEMPLATE").ok()?;
    WorldTemplate::ALL.into_iter().find(|t| t.label() == want)
}

/// Camera state recorded around synthetic mouse input, to prove the drag and
/// wheel paths move the camera through Bevy's real input pipeline.
#[derive(Resource, Default)]
struct InputProbe { before: Option<(Vec3, f32)>, drag_ok: Option<bool>, wheel_ok: Option<bool> }

/// Frames 140-160: press right button and move; frames 170-172: wheel up.
fn inject_mouse(
    script: Res<CaptureScript>,
    window: Query<Entity, With<PrimaryWindow>>,
    mut buttons: MessageWriter<MouseButtonInput>,
    mut motion: MessageWriter<MouseMotion>,
    mut wheel: MessageWriter<MouseWheel>,
) {
    let Ok(window) = window.single() else { return; };
    let f = script.frame;
    let press = |state| MouseButtonInput { button: MouseButton::Right, state, window };
    match f {
        140 => { buttons.write(press(ButtonState::Pressed)); }
        141..=150 => { motion.write(MouseMotion { delta: Vec2::new(20.0, 0.0) }); }
        151 => { buttons.write(press(ButtonState::Released)); }
        170..=172 => { wheel.write(MouseWheel { unit: MouseScrollUnit::Line, x: 0.0, y: 1.0, window, phase: TouchPhase::Moved }); }
        _ => {}
    }
}

fn run_capture(
    mut commands: Commands,
    mut script: ResMut<CaptureScript>,
    mut state: ResMut<ViewerState>,
    mut camera: Query<&mut Transform, With<WorldCamera>>,
    mut probe: ResMut<InputProbe>,
    mut exit: MessageWriter<AppExit>,
) {
    script.frame += 1;
    let shot = |commands: &mut Commands, dir: &std::path::Path, name: &str| {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(dir.join(name)));
    };
    match script.frame {
        // Set a fire in the nearest good fuel and release a flood nearby, then let time run.
        10 => {
            let home = state.sim.residents.first().map(|r| r.position);
            if let (Some(home), Some(t)) = (home, state.sim.terrain.clone()) {
                let spot = (0..t.biome.len())
                    .filter(|&i| sim_core::terrain::hazards::base_fuel(t.biome[i]) >= 0.7)
                    .map(|i| t.tile_center(i % t.width, i / t.width))
                    .filter(|p| p.distance(home) > 40.0)
                    .min_by(|a, b| a.distance(home).total_cmp(&b.distance(home)));
                if let Some(p) = spot { state.sim.ignite_at(p, 14.0, 1.0); }
                state.sim.flood_at(sim_core::world::Position { x: home.x - 60.0, y: home.y - 30.0 }, 28.0, 1.0);
            }
            state.speed = 20.0;
            // Open a character card for the shot.
            state.selected = state.sim.residents.first().map(|r| crate::Selected::Resident(r.id));
        }
        40 => shot(&mut commands, &script.dir, "village.png"),
        41 => state.speed = 1.0,
        42 => {
            if let (Ok(mut t), Some(m)) = (camera.single_mut(), state.sim.terrain.as_deref()) {
                let c = crate::terrain_view::to_view(Some(m), sim_core::world::Position { x: 0.0, y: 0.0 });
                t.translation.x = c.x;
                t.translation.y = c.y;
                let s = crate::terrain_view::max_scale(Some(m));
                t.scale = Vec3::new(s, s, 1.0);
            }
        }
        60 => shot(&mut commands, &script.dir, "world.png"),
        61 => state.map_mode = MapMode::Biome,
        80 => shot(&mut commands, &script.dir, "biome.png"),
        81 => state.map_mode = MapMode::Terrain,
        // Exercise the new-world path: old chunks must be replaced, not stacked.
        82 => {
            let i = WorldTemplate::ALL.iter().position(|t| *t == state.template).unwrap_or(0);
            state.template = WorldTemplate::ALL[(i + 1) % WorldTemplate::ALL.len()];
            state.rebuild_world();
            // Show a populated dock category in the last shot.
            state.tool = crate::GodTool::Fire;
        }
        100 => shot(&mut commands, &script.dir, "rebuilt.png"),
        135 => { if let Ok(t) = camera.single() { probe.before = Some((t.translation, t.scale.x)); } }
        160 => if let (Ok(t), Some((p, _))) = (camera.single(), probe.before) {
            // Dragging right moves the view left: camera x must decrease.
            probe.drag_ok = Some(t.translation.x < p.x - 1.0);
            probe.before = Some((t.translation, t.scale.x));
        },
        180 => if let (Ok(t), Some((_, s))) = (camera.single(), probe.before) {
            probe.wheel_ok = Some(t.scale.x < s * 0.99);
        },
        181 => {
            info!("INPUT PROBE right-drag pans: {:?}  wheel zooms in: {:?}", probe.drag_ok, probe.wheel_ok);
            exit.write(AppExit::Success);
        }
        _ => {}
    }
}
