use bevy::prelude::*;

use crate::{graph::{Dot, Neighbors}, WIN_SIZE};

pub const REPEL_STRENGTH: f32 = 1000.;
pub const SPRING_COEFFICIENT: f32 = 0.012;
pub const SPRING_RESTING_LENGTH: f32 = 50.;
pub const WALL_REPEL_STRENGTH: f32 = 1000.;
pub const VEL_DAMPENING: f32 = 0.7;
pub const VEL_CAP: f32 = 50.;
pub const ACC_DAMPENING: f32 = 0.85;
pub const ACC_CAP: f32 = 10.;

// Runtime configuration for above starting constants
#[derive(Resource)]
pub struct PhysicsConfig {
    pub repel_strength: f32,
    pub spring_coefficient: f32,
    pub spring_resting_length: f32,
    pub wall_repel_strength: f32,
    pub vel_dampening: f32,
    pub vel_cap: f32,
    pub acc_dampening: f32,
    pub acc_cap: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            repel_strength: REPEL_STRENGTH,
            spring_coefficient: SPRING_COEFFICIENT,
            spring_resting_length: SPRING_RESTING_LENGTH,
            wall_repel_strength: WALL_REPEL_STRENGTH,
            vel_dampening: VEL_DAMPENING,
            vel_cap: VEL_CAP,
            acc_dampening: ACC_DAMPENING,
            acc_cap: ACC_CAP,
        }
    }
}


#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct Acceleration(pub Vec3);

pub fn apply_velocity(
    mut q: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>
) {
    for (mut tf, Velocity(vel)) in q.iter_mut() {
        tf.translation += *vel * time.delta_seconds() * 300.0;
    }
}

pub fn apply_acceleration(
    mut q: Query<(&mut Velocity, &Acceleration)>,
    time: Res<Time>
) {
    for (mut vel, Acceleration(accel)) in q.iter_mut() {
        vel.0 += *accel * time.delta_seconds();
    }
}

pub fn apply_force_between_dots(
    mut q: Query<(&mut Acceleration, &Transform), With<Dot>>,
    physics_config: Res<PhysicsConfig>,
) {
    let mut iter = q.iter_combinations_mut();
    while let Some([(mut accel, tf), (mut accel2, tf2)]) = iter.fetch_next() {
        let d = Vec3::distance_squared(tf.translation, tf2.translation);
        let repel_force = if d == 0. {
            0.
        } else {   
            physics_config.repel_strength / d
        };
        let a_to_b = (tf2.translation - tf.translation).normalize_or_zero();

        accel.0 -= a_to_b * repel_force;
        accel2.0 += a_to_b * repel_force;
    }
}

pub fn apply_attraction_between_edges(
    mut q: Query<(&mut Acceleration, &Transform, &Neighbors, Entity), With<Dot>>,
    physics_config: Res<PhysicsConfig>,
) {
    let mut pairs = Vec::new();
    for (_, _, neighbors, eid) in q.iter() {
        for neighbor_eid in neighbors.neighbors.iter().cloned() {
            pairs.push((eid, neighbor_eid));
        }
    }

    for (eid, neighbor_eid) in pairs {
        let [mut e1, mut e2] = q.get_many_mut([eid, neighbor_eid]).unwrap();

        let d = f32::max(Vec3::distance(e1.1.translation, e2.1.translation) - physics_config.spring_resting_length, 0.);
        let spring_force = physics_config.spring_coefficient * d;

        let a_to_b = (e2.1.translation - e1.1.translation).normalize();

        e1.0.0 += a_to_b * spring_force;
        e2.0.0 -= a_to_b * spring_force;
    }
}

pub fn apply_force_between_dots_and_walls(
    mut q: Query<(&mut Acceleration, &Transform), With<Dot>>,
    physics_config: Res<PhysicsConfig>
) {
    for (mut accel, tf) in q.iter_mut() {
        let right_wall_force = physics_config.wall_repel_strength / f32::powi(f32::max(WIN_SIZE.0 - tf.translation.x, 0.0001), 2);
        let left_wall_force = physics_config.wall_repel_strength / f32::powi(f32::max(tf.translation.x, 0.0001), 2);
        let bottom_wall_force = physics_config.wall_repel_strength / f32::powi(f32::max(WIN_SIZE.1 - tf.translation.y, 0.0001), 2);
        let top_wall_force = physics_config.wall_repel_strength / f32::powi(f32::max(tf.translation.y, 0.0001), 2);

        // apply forces for each wall
        accel.0.x += left_wall_force - right_wall_force;
        accel.0.y += top_wall_force - bottom_wall_force;
    }
}

pub fn vel_dampen(
    mut q: Query<&mut Velocity>,
    physics_config: Res<PhysicsConfig>
) {
    for mut vel in q.iter_mut() {
        vel.0 *= physics_config.vel_dampening;
        if vel.0.length_squared() > physics_config.vel_cap * physics_config.vel_cap {
            vel.0 = vel.0.normalize() * physics_config.vel_cap;
        }
    }
}

pub fn accel_dampen(
    mut q: Query<&mut Acceleration>,
    physics_config: Res<PhysicsConfig>
) {
    for mut acc in q.iter_mut() {
        acc.0 *= physics_config.acc_dampening;
        if acc.0.length_squared() > physics_config.acc_cap * physics_config.acc_cap {
            acc.0 = acc.0.normalize() * physics_config.acc_cap;
        }
    }
}
