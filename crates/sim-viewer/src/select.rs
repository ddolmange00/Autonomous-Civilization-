//! StarCraft-style box selection: with the Inspect tool, left-drag a rectangle
//! to select every living resident inside it. A plain click still picks one
//! thing (handled by `world_click`).
use bevy::{prelude::*, window::PrimaryWindow};

use crate::{terrain_view, GodTool, Selected, ViewerState, WorldCamera};

/// Pixels the cursor must travel before a press becomes a box drag.
const DRAG_THRESHOLD: f32 = 6.0;

#[derive(Resource, Default)]
pub struct DragSelect { start: Option<Vec2>, active: bool }

#[derive(Component)]
pub struct SelectBox;

pub fn spawn_box(mut commands: Commands) {
    commands.spawn((
        Node { position_type: PositionType::Absolute, border: UiRect::all(px(1)), ..default() },
        BackgroundColor(Color::srgba(0.55, 0.9, 0.55, 0.12)),
        BorderColor::all(Color::srgb(0.55, 0.95, 0.55)),
        Visibility::Hidden, SelectBox,
    ));
}

pub fn box_select(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<WorldCamera>>,
    ui: Query<&Interaction>,
    mut state: ResMut<ViewerState>,
    mut drag: ResMut<DragSelect>,
    mut rect: Query<(&mut Node, &mut Visibility), With<SelectBox>>,
) {
    let Ok(w) = window.single() else { return; };
    let cursor = w.cursor_position();
    let Ok((mut node, mut vis)) = rect.single_mut() else { return; };
    if buttons.just_pressed(MouseButton::Left) {
        let over_ui = ui.iter().any(|i| *i != Interaction::None);
        drag.start = if state.tool == GodTool::Inspect && !state.monster_lab && !over_ui { cursor } else { None };
        drag.active = false;
    }
    let (Some(start), Some(now)) = (drag.start, cursor) else {
        *vis = Visibility::Hidden;
        return;
    };
    if buttons.pressed(MouseButton::Left) {
        if now.distance(start) > DRAG_THRESHOLD { drag.active = true; }
        if drag.active {
            let (min, max) = (start.min(now), start.max(now));
            node.left = px(min.x);
            node.top = px(min.y);
            node.width = px(max.x - min.x);
            node.height = px(max.y - min.y);
            *vis = Visibility::Visible;
        }
        return;
    }
    // Released.
    if drag.active {
        let (min, max) = (start.min(now), start.max(now));
        if let Ok((cam, global)) = camera.single() {
            let t = state.sim.terrain.as_deref();
            let picked: Vec<u64> = state.sim.residents.iter().filter(|r| r.health > 0.0).filter(|r| {
                let v = terrain_view::to_view(t, r.position) + Vec2::new(0.0, 8.0);
                cam.world_to_viewport(global, v.extend(0.0)).map(|p| p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y).unwrap_or(false)
            }).map(|r| r.id).collect();
            state.selected = match picked.len() {
                0 => None,
                1 => Some(Selected::Resident(picked[0])),
                _ => Some(Selected::Group(picked)),
            };
        }
    }
    drag.start = None;
    drag.active = false;
    *vis = Visibility::Hidden;
}
