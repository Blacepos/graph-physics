use bevy::prelude::*;
use rand::Rng;

use crate::{graph::{Dot, Neighbors, Partner}, physics::{Acceleration, Velocity}, Randomness, WIN_SIZE};

pub const NUMBER_OF_DOTS: usize = 200;
const SEPARATION_ON_GRID: f32 = 40.;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum Phases {
    #[default]
    Init,
    JustDots,
    Graph,
    DisconnectedEdges,
    PointingSegments
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SpawnMethod {
    Grid,
    Random,
}

pub fn clear_dots(
    mut commands: Commands,
    query: Query<Entity, With<Dot>>,
) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}

pub fn spawn_dots(
    next_state: ResMut<NextState<Phases>>,
    commands: Commands,
    randomness: ResMut<Randomness>,
    spawn_method: Res<SpawnMethod>,
) {
    match *spawn_method {
        SpawnMethod::Grid => spawn_dots_grid(next_state, commands),
        SpawnMethod::Random => spawn_dots_random(next_state, commands, randomness),
    }
}

pub fn spawn_dots_random(
    mut next_state: ResMut<NextState<Phases>>,
    mut commands: Commands,
    mut randomness: ResMut<Randomness>,
) {
    for _ in 0..NUMBER_OF_DOTS {
        let x = randomness.0.gen_range(100.0..WIN_SIZE.0-100.0);
        let y = randomness.0.gen_range(100.0..WIN_SIZE.1-100.0);
        commands.spawn((
            Dot,
            Transform::from_xyz(x, y, 0.),
            Neighbors { neighbors: Vec::new() },
            Partner { partner: None },
            Velocity(Vec3::ZERO),
            Acceleration(Vec3::ZERO),
        ));
    }

    next_state.set(Phases::JustDots);
}

pub fn spawn_dots_grid(
    mut next_state: ResMut<NextState<Phases>>,
    mut commands: Commands,
) {
    let number_of_columns = (NUMBER_OF_DOTS as f32).sqrt() as usize;
    let grid_start_x = WIN_SIZE.0 / 2. - (number_of_columns as f32) * SEPARATION_ON_GRID / 2.;
    let grid_start_y = WIN_SIZE.1 / 2. - (number_of_columns as f32) * SEPARATION_ON_GRID / 2.;
    for i in 0..NUMBER_OF_DOTS {
        let grid_x = i % number_of_columns;
        let grid_y = i / number_of_columns;
        let x = grid_start_x + grid_x as f32 * SEPARATION_ON_GRID;
        let y = grid_start_y + grid_y as f32 * SEPARATION_ON_GRID;
        commands.spawn((
            Dot,
            Transform::from_xyz(x, y, 0.),
            Neighbors { neighbors: Vec::new() },
            Partner { partner: None },
            Velocity(Vec3::ZERO),
            Acceleration(Vec3::ZERO),
        ));
    }

    next_state.set(Phases::JustDots);
}

pub fn test_transitions(
    mut next_state: ResMut<NextState<Phases>>,
    time: Res<Time>
) {
    let t = time.elapsed_seconds();
    if t > 0. {
        next_state.set(Phases::JustDots);
    }
    if t > 0.1 {
        next_state.set(Phases::Graph);
    }
    // if t > 16. {
    //     next_state.set(Phases::DisconnectedEdges);
    // }
    // if t > 20. {
    //     next_state.set(Phases::Graph);
    // }
}