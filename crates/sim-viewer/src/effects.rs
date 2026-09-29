//! Short-lived particles that make world physics readable: smoke and embers
//! from burning tiles, rain streaks, glints on floodwater, earthquake dust and
//! camera shake. Purely visual; every effect is driven by simulation state.
use bevy::prelude::*;
use sim_core::{events::WorldEventKind, terrain::TileMap};

use crate::{sprites::{SpriteBank, SpriteKey}, terrain_view, ViewerState, WorldCamera};

const MAX_PARTICLES: usize = 2500;
const PARTICLE_Z: f32 = 70.0;

#[derive(Component)]
pub struct Particle { vel: Vec2, life: f32, max_life: f32, grow: f32, alpha: f32 }

#[derive(Resource, Default)]
pub struct Shake { applied: Vec2 }

#[derive(Resource)]
pub struct FxRng(u64);
impl Default for FxRng { fn default() -> Self { Self(0x2545_F491_4F6C_DD1D) } }
impl FxRng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, a: f32, b: f32) -> f32 { a + (b - a) * self.next() }
}

fn tile_view(t: &TileMap, i: usize) -> Vec2 {
    terrain_view::to_view(Some(t), t.tile_center(i % t.width, i / t.width))
}

pub fn spawn_effects(
    mut commands: Commands,
    state: Res<ViewerState>,
    time: Res<Time>,
    mut rng: ResMut<FxRng>,
    mut bank: ResMut<SpriteBank>,
    mut images: ResMut<Assets<Image>>,
    particles: Query<(), With<Particle>>,
) {
    if state.paused { return; }
    let Some(t) = state.sim.terrain.as_deref() else { return; };
    let budget = MAX_PARTICLES.saturating_sub(particles.iter().count());
    if budget == 0 { return; }
    let dt = time.delta_secs().min(0.1);
    let (disc, _) = bank.get(SpriteKey::Disc, &mut images);
    let mut spawned = 0usize;
    let mut spawn = |commands: &mut Commands, at: Vec2, size: Vec2, colour: Color, vel: Vec2, life: f32, grow: f32, round: bool| {
        if spawned >= budget { return; }
        spawned += 1;
        let sprite = if round {
            Sprite { image: disc.clone(), color: colour, custom_size: Some(size), ..default() }
        } else {
            Sprite::from_color(colour, size)
        };
        let alpha = colour.alpha();
        commands.spawn((sprite, Transform::from_xyz(at.x, at.y, PARTICLE_Z), Particle { vel, life, max_life: life, grow, alpha }, crate::WorldDynamic));
    };

    if let Some(hz) = state.sim.hazards.as_ref() {
        let burning = hz.burning();
        if !burning.is_empty() {
            let wind = Vec2::new(hz.wind.0 - hz.wind.1, -(hz.wind.0 + hz.wind.1) * 0.5) * 10.0;
            let puffs = (burning.len().min(500) as f32 * dt * 1.2).ceil() as usize;
            for _ in 0..puffs {
                let i = burning[(rng.next() * burning.len() as f32) as usize % burning.len()] as usize;
                let at = tile_view(t, i) + Vec2::new(rng.range(-4.0, 4.0), 14.0);
                let grey = rng.range(0.18, 0.32);
                spawn(&mut commands, at, Vec2::splat(6.0), Color::srgba(grey, grey, grey, 0.45), wind + Vec2::new(rng.range(-3.0, 3.0), rng.range(16.0, 26.0)), rng.range(1.6, 2.8), 9.0, true);
                if rng.next() < 0.35 {
                    spawn(&mut commands, at, Vec2::splat(1.5), Color::srgba(1.0, 0.62, 0.2, 0.95), wind * 2.0 + Vec2::new(rng.range(-8.0, 8.0), rng.range(30.0, 55.0)), rng.range(0.4, 0.9), 0.0, false);
                }
            }
        }
        let wet = hz.wet();
        if !wet.is_empty() {
            let glints = (wet.len().min(800) as f32 * dt * 0.8).ceil() as usize;
            for _ in 0..glints {
                let i = wet[(rng.next() * wet.len() as f32) as usize % wet.len()] as usize;
                if hz.water[i] < 0.1 { continue; }
                let at = tile_view(t, i) + Vec2::new(rng.range(-6.0, 6.0), rng.range(-2.0, 2.0));
                spawn(&mut commands, at, Vec2::new(3.0, 1.0), Color::srgba(0.85, 0.95, 1.0, 0.8), Vec2::new(rng.range(-4.0, 4.0), 0.0), rng.range(0.4, 0.9), 0.0, false);
            }
        }
    }

    for ev in state.sim.events.iter().filter(|e| e.active(state.sim.year)) {
        let centre = terrain_view::to_view(Some(t), ev.position);
        let half = terrain_view::ground_ellipse(Some(t), ev.radius) * 0.5;
        let area = (half.x * half.y).max(1.0);
        let point = |rng: &mut FxRng| {
            let a = rng.next() * std::f32::consts::TAU;
            let r = rng.next().sqrt();
            centre + Vec2::new(a.cos() * half.x * r, a.sin() * half.y * r)
        };
        match ev.kind {
            WorldEventKind::Rain | WorldEventKind::Storm => {
                let drops = ((area * dt * 0.04).ceil() as usize).min(90);
                for _ in 0..drops {
                    let at = point(&mut rng) + Vec2::new(0.0, 40.0);
                    spawn(&mut commands, at, Vec2::new(1.0, 7.0), Color::srgba(0.72, 0.82, 1.0, 0.55), Vec2::new(-35.0, -260.0), 0.16, 0.0, false);
                }
            }
            WorldEventKind::Earthquake => {
                let puffs = ((area * dt * 0.01).ceil() as usize).min(30);
                for _ in 0..puffs {
                    let at = point(&mut rng);
                    spawn(&mut commands, at, Vec2::splat(5.0), Color::srgba(0.55, 0.45, 0.32, 0.5), Vec2::new(rng.range(-10.0, 10.0), rng.range(4.0, 12.0)), rng.range(0.8, 1.5), 7.0, true);
                }
            }
            _ => {}
        }
    }
}

pub fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<ViewerState>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = if state.paused { 0.0 } else { time.delta_secs().min(0.1) };
    for (e, mut p, mut tf, mut sprite) in &mut particles {
        p.life -= dt;
        if p.life <= 0.0 { commands.entity(e).despawn(); continue; }
        tf.translation.x += p.vel.x * dt;
        tf.translation.y += p.vel.y * dt;
        if p.grow > 0.0 {
            if let Some(size) = sprite.custom_size.as_mut() { *size += Vec2::splat(p.grow * dt); }
        }
        let k = (p.life / p.max_life).clamp(0.0, 1.0);
        sprite.color.set_alpha(p.alpha * k);
    }
}

/// Earthquakes on screen shake the camera; the offset is removed again each frame.
pub fn camera_shake(
    state: Res<ViewerState>,
    mut shake: ResMut<Shake>,
    mut rng: ResMut<FxRng>,
    mut camera: Query<&mut Transform, With<WorldCamera>>,
) {
    let Ok(mut tf) = camera.single_mut() else { return; };
    tf.translation.x -= shake.applied.x;
    tf.translation.y -= shake.applied.y;
    let strength = state.sim.events.iter()
        .filter(|e| e.kind == WorldEventKind::Earthquake && e.active(state.sim.year))
        .map(|e| e.intensity).fold(0.0f32, f32::max);
    shake.applied = if strength > 0.0 && !state.paused {
        Vec2::new(rng.range(-1.0, 1.0), rng.range(-1.0, 1.0)) * 3.5 * strength * tf.scale.x
    } else { Vec2::ZERO };
    tf.translation.x += shake.applied.x;
    tf.translation.y += shake.applied.y;
}
