//! Isometric terrain renderer, world<->view projection and the map camera.
//!
//! View space is Bevy world space holding the full-resolution isometric image:
//! x right, y up (the iso screen y flipped). Terrain is two layers of baked
//! chunk images: a coarse base covering the whole map, and a detail layer
//! streamed in only for chunks on screen when zoomed in.
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};
use sim_core::{
    terrain::{iso, palette::MapMode, TileMap, TILE_SIZE},
    world::Position,
};

use crate::{ViewerState, WorldCamera};

/// Base layer resolution (pixels per tile edge in the baked image).
const BASE_TILE_W: u32 = 2;
/// Detail chunks baked per frame, so zooming never stalls a frame.
const BAKE_BUDGET: usize = 6;
const BASE_Z: f32 = -300.0;
const DETAIL_Z: f32 = -200.0;

// ---------- projection ----------

/// View-space position of a sim position standing on the terrain.
pub fn to_view(t: Option<&TileMap>, p: Position) -> Vec2 {
    match t {
        Some(m) => { let (sx, sy) = iso::world_to_screen(m, p); Vec2::new(sx, -sy) }
        None => Vec2::new(p.x, p.y),
    }
}

/// Sim position of the terrain drawn at a view-space point.
pub fn from_view(t: Option<&TileMap>, v: Vec2) -> Position {
    match t {
        Some(m) => iso::screen_to_world(m, v.x, -v.y),
        None => Position { x: v.x, y: v.y },
    }
}

/// Draw order for things standing on the map: nearer the viewer (larger
/// tile x+y) draws on top. Range roughly 1..51.
pub fn depth(t: Option<&TileMap>, p: Position) -> f32 {
    match t {
        Some(m) => { let (tx, ty) = iso::world_to_tile(m, p); 1.0 + (tx + ty) / (m.width + m.height) as f32 * 50.0 }
        None => 1.0,
    }
}

/// View-space size of a ground disc of `radius` world units (an iso ellipse).
pub fn ground_ellipse(t: Option<&TileMap>, radius: f32) -> Vec2 {
    match t {
        Some(_) => {
            let r_tiles = radius / TILE_SIZE;
            Vec2::new(r_tiles * iso::FULL_TILE_W, r_tiles * iso::FULL_TILE_H) * std::f32::consts::SQRT_2
        }
        None => Vec2::splat(radius * 2.0),
    }
}

// ---------- terrain layers ----------

#[derive(Component)]
pub struct TerrainChunk { cx: usize, cy: usize, tile_w: u32, revision: u32, mode: MapMode, generation: u64 }

fn chunk_image(b: iso::BakedChunk) -> Image {
    let mut img = Image::new(
        Extent3d { width: b.width, height: b.height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        b.rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    img.sampler = ImageSampler::nearest();
    img
}

fn chunk_transform(cx: usize, cy: usize, z: f32) -> (Transform, Vec2) {
    let ((ox, oy), (w, h)) = iso::chunk_bounds(cx, cy);
    // Chunks further south-east overlap those behind them.
    let z = z + (cx + cy) as f32 * 0.01;
    (Transform::from_xyz(ox + w * 0.5, -(oy + h * 0.5), z), Vec2::new(w, h))
}

/// Visible view-space rectangle (min, max) with a margin.
fn view_rect(cam: &Transform, window: &Window) -> (Vec2, Vec2) {
    let half = Vec2::new(window.width(), window.height()) * 0.5 * cam.scale.x * 1.15;
    let c = cam.translation.truncate();
    (c - half, c + half)
}

fn chunk_visible(cx: usize, cy: usize, rect: (Vec2, Vec2)) -> bool {
    let ((ox, oy), (w, h)) = iso::chunk_bounds(cx, cy);
    let (min, max) = (Vec2::new(ox, -(oy + h)), Vec2::new(ox + w, -oy));
    min.x <= rect.1.x && max.x >= rect.0.x && min.y <= rect.1.y && max.y >= rect.0.y
}

/// Detail resolution wanted at the current zoom (0 = base layer is enough).
fn detail_tile_w(scale: f32) -> u32 {
    let px_per_tile = iso::FULL_TILE_W / scale.max(1e-3);
    if px_per_tile >= 7.0 { 16 } else if px_per_tile >= 2.5 { 8 } else { 0 }
}

pub fn sync_terrain(
    mut commands: Commands,
    state: Res<ViewerState>,
    mut images: ResMut<Assets<Image>>,
    mut chunks: Query<(Entity, &mut TerrainChunk, &Sprite)>,
    camera: Query<&Transform, With<WorldCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Some(t) = state.sim.terrain.as_deref() else {
        for (e, _, _) in &chunks { commands.entity(e).despawn(); }
        return;
    };
    let (Ok(cam), Ok(win)) = (camera.single(), window.single()) else { return; };
    let mode = state.map_mode;
    let hz = state.sim.hazards.as_ref();
    // A chunk redraws when its tiles or the fire/water on them change.
    let revision = |cx: usize, cy: usize| t.chunk_revision(cx, cy).wrapping_add(hz.map(|h| h.chunk_revision(cx, cy)).unwrap_or(0));
    let rect = view_rect(cam, win);
    let want_detail = detail_tile_w(cam.scale.x);
    let (cw, ch) = (t.chunks_x(), t.chunks_y());
    let mut have_base = false;
    let mut have_detail = vec![false; cw * ch];
    let mut rebaked = 0;
    for (e, mut c, sprite) in &mut chunks {
        if c.generation != state.world_generation {
            commands.entity(e).despawn();
            continue;
        }
        if c.tile_w != BASE_TILE_W {
            // Detail chunks leave when off screen or at the wrong resolution.
            if c.tile_w != want_detail || !chunk_visible(c.cx, c.cy, rect) {
                commands.entity(e).despawn();
                continue;
            }
            have_detail[c.cy * cw + c.cx] = true;
        } else {
            have_base = true;
        }
        let rev = revision(c.cx, c.cy);
        if (c.revision != rev || c.mode != mode) && (c.tile_w == BASE_TILE_W || rebaked < BAKE_BUDGET) {
            if c.tile_w != BASE_TILE_W { rebaked += 1; }
            let baked = iso::bake_chunk(t, hz, c.cx, c.cy, mode, c.tile_w);
            if let Some(mut img) = images.get_mut(&sprite.image) { img.data = Some(baked.rgba); }
            c.revision = rev;
            c.mode = mode;
        }
    }
    if !have_base {
        for cy in 0..ch {
            for cx in 0..cw {
                let handle = images.add(chunk_image(iso::bake_chunk(t, hz, cx, cy, mode, BASE_TILE_W)));
                let (tf, size) = chunk_transform(cx, cy, BASE_Z);
                commands.spawn((
                    Sprite { image: handle, custom_size: Some(size), ..default() }, tf,
                    TerrainChunk { cx, cy, tile_w: BASE_TILE_W, revision: revision(cx, cy), mode, generation: state.world_generation },
                ));
            }
        }
    }
    if want_detail == 0 { return; }
    // Stream missing detail chunks, nearest to the view centre first.
    let centre = cam.translation.truncate();
    let mut missing: Vec<(f32, usize, usize)> = Vec::new();
    for cy in 0..ch {
        for cx in 0..cw {
            if have_detail[cy * cw + cx] || !chunk_visible(cx, cy, rect) { continue; }
            let (tf, _) = chunk_transform(cx, cy, 0.0);
            missing.push((tf.translation.truncate().distance(centre), cx, cy));
        }
    }
    missing.sort_by(|a, b| a.0.total_cmp(&b.0));
    for &(_, cx, cy) in missing.iter().take(BAKE_BUDGET.saturating_sub(rebaked)) {
        let handle = images.add(chunk_image(iso::bake_chunk(t, hz, cx, cy, mode, want_detail)));
        let (tf, size) = chunk_transform(cx, cy, DETAIL_Z);
        commands.spawn((
            Sprite { image: handle, custom_size: Some(size), ..default() }, tf,
            TerrainChunk { cx, cy, tile_w: want_detail, revision: revision(cx, cy), mode, generation: state.world_generation },
        ));
    }
}

// ---------- camera ----------

/// Largest zoom-out: the whole map fits the window width.
pub fn max_scale(t: Option<&TileMap>) -> f32 {
    t.map(|m| (m.width.max(m.height) as f32 * iso::FULL_TILE_W / 1400.0).max(1.0)).unwrap_or(4.0)
}
const MIN_SCALE: f32 = 0.12;

/// Wheel zooms toward the cursor; right or middle drag pans; holding the cursor
/// at a window edge scrolls, RTS-style. Left drag is box selection (select.rs).
pub fn map_camera(
    scroll: Res<AccumulatedMouseScroll>,
    motion: Res<AccumulatedMouseMotion>,
    buttons: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    window: Query<&Window, With<PrimaryWindow>>,
    state: Res<ViewerState>,
    mut camera: Query<(&Camera, &GlobalTransform, &mut Transform), With<WorldCamera>>,
) {
    let Ok((cam, global, mut t)) = camera.single_mut() else { return; };
    let max = max_scale(state.sim.terrain.as_deref());
    if scroll.delta.y != 0.0 && !state.monster_lab {
        let steps = match scroll.unit { MouseScrollUnit::Line => scroll.delta.y, MouseScrollUnit::Pixel => scroll.delta.y / 60.0 };
        let old = t.scale.x;
        let new = (old * 0.85f32.powf(steps)).clamp(MIN_SCALE, max);
        // Keep the view point under the cursor fixed while zooming.
        if let Some(cursor) = window.single().ok().and_then(|w| w.cursor_position()) {
            if let Ok(anchor) = cam.viewport_to_world_2d(global, cursor) {
                let moved = anchor + (t.translation.truncate() - anchor) * (new / old);
                t.translation.x = moved.x;
                t.translation.y = moved.y;
            }
        }
        t.scale = Vec3::new(new, new, 1.0);
    }
    if buttons.pressed(MouseButton::Right) || buttons.pressed(MouseButton::Middle) {
        let d = motion.delta * t.scale.x;
        t.translation.x -= d.x;
        t.translation.y += d.y;
    }
    if let Ok(w) = window.single() {
        if let (true, Some(c)) = (w.focused, w.cursor_position()) {
            const EDGE: f32 = 8.0;
            let mut dir = Vec2::ZERO;
            if c.x <= EDGE { dir.x -= 1.0; }
            if c.x >= w.width() - EDGE { dir.x += 1.0; }
            if c.y <= EDGE { dir.y += 1.0; }
            if c.y >= w.height() - EDGE { dir.y -= 1.0; }
            if dir != Vec2::ZERO && !state.monster_lab {
                let step = dir.normalize() * 900.0 * time.delta_secs() * t.scale.x;
                t.translation.x += step.x;
                t.translation.y += step.y;
            }
        }
    }
    t.scale.x = t.scale.x.clamp(MIN_SCALE, max);
    t.scale.y = t.scale.x;
}

/// Short description of the tile under the cursor for the HUD.
pub fn cursor_tile_label(t: &TileMap, view: Vec2) -> Option<String> {
    let p = from_view(Some(t), view);
    let (x, y) = t.tile_at(p)?;
    let i = t.index(x, y);
    Some(format!(
        "TILE {x},{y}  {:?} / {}  {}m  {}C  wet {:.0}%{}",
        t.ground[i], t.biome[i].label(), t.height_m[i], t.temperature_c[i],
        t.moisture[i] as f32 / 2.55,
        if t.river[i] > 0 { "  river" } else { "" },
    ))
}
