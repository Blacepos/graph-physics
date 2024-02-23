#![feature(iterator_try_collect)]
#![windows_subsystem = "windows"]
use bevy::{log::LogPlugin, prelude::*, window::{PresentMode, PrimaryWindow, WindowResolution}};
use graph::{compute_disjoint_pairs, compute_neighbors, GraphSpawnConfig};
use phases::{clear_dots, spawn_dots, test_transitions, Phases, SpawnMethod};
use physics::{accel_dampen, apply_acceleration, apply_attraction_between_edges, apply_force_between_dots, apply_force_between_dots_and_walls, apply_velocity, vel_dampen, PhysicsConfig};
use rand::{rngs::StdRng, SeedableRng};
use render::{render_dots, render_graph_edges, render_partners};
use bevy_egui::EguiPlugin;
use ui::ui_tweak_panel;

mod phases;
mod graph;
mod render;
mod physics;
mod ui;

const WIN_SIZE: (f32, f32) = (1280.0, 720.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
            .set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                filter: "warn,profile_website=debug".into(),
                ..Default::default()
            })
            .set(WindowPlugin {
                    primary_window: Some(Window {
                    resolution: WindowResolution::new(WIN_SIZE.0, WIN_SIZE.1),
                    resizable: false,
                    present_mode: PresentMode::AutoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            })
        )
        .add_plugins(EguiPlugin)
        .insert_state(Phases::Init)
        .insert_resource(MousePosition(Vec2::ZERO))
        .insert_resource(Randomness(StdRng::seed_from_u64(69)))
        .insert_resource(PhysicsConfig::default())
        .insert_resource(SpawnMethod::Random)
        .insert_resource(GraphSpawnConfig::default())
        .add_systems(Startup, startup)
        // Phase transitions
        .add_systems(OnEnter(Phases::Init), (
            clear_dots,
            spawn_dots.after(clear_dots),
        ))
        .add_systems(OnEnter(Phases::Graph), compute_neighbors)
        .add_systems(OnEnter(Phases::DisconnectedEdges), compute_disjoint_pairs)
        // Always run inside phase
        .add_systems(Update, render_dots.run_if(in_state(Phases::JustDots)))
        .add_systems(Update, (render_dots, render_graph_edges).run_if(in_state(Phases::Graph)))
        .add_systems(Update, render_partners.run_if(in_state(Phases::DisconnectedEdges)))
        // Always run
        .add_systems(Update, (
            update_mouse,
            test_transitions,
            apply_acceleration,
            apply_velocity,
            apply_force_between_dots,
            apply_attraction_between_edges,
            vel_dampen,
            accel_dampen,
            ui_tweak_panel,
            apply_force_between_dots_and_walls
        ))
        .run();
}

fn startup(
    mut commands: Commands,
    mut gizmos_config_store: ResMut<GizmoConfigStore>,
) {
    // I'm more comfortable with top-down, left-right coordinates
    let cam_bundle = Camera2dBundle {
        transform: Transform::IDENTITY
            .with_translation(Vec3::new(WIN_SIZE.0 / 2., WIN_SIZE.1 / 2., 0.))
            .with_scale(Vec3::new(1., -1., 1.)),
        ..Default::default()
    };
    commands.spawn((cam_bundle, MainCamera));

    let (gizmos_config, _) = gizmos_config_store.config_mut::<DefaultGizmoConfigGroup>();
    gizmos_config.line_width = 1.0;
}

/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

#[derive(Resource, Default)]
struct MousePosition(Vec2);

#[derive(Resource)]
struct Randomness(StdRng);

fn update_mouse(
    mut mouse: ResMut<MousePosition>,
    window_q: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_q.single();
    if let Some(world_position) = window.cursor_position()
    {
        mouse.0 = world_position;
    }
}

/*
// phase 1 (just dots)
1. Dots spawn randomly on load
2. Every dot computes their neighbors (stored in component data)
    - Fastest: For every dot, check N random dots and make neighbor if distance less than D
    - For every dot, iterate through every other dot and make neighbor if distance less than D
    - Slowest: For every dot, sort every other dot by distance and pick N nearest (probably don't use this. keep in mind all the dots are roughly the same distance apart anyways)
3. Dots repel each other
4. Mouse repels dots

// phase 2 (dots -> graph)
1. Every dot gradually extends a line going from itself to its neighbors
2. Dots fade out
Undo:
1. Dots fade in
2. Dots gradually retract their lines


// phase 3 (graph -> disconnected edges)
1. dots no longer repeled by mouse (fixes a potential issue)
2. graph -> segments algorithm:
    - elimination algorithm (iterate all: pick first neighbor without a visible edge)
    - Hard to undo: every dot replicates for every neighbor and becomes half of the edge
3. Edges are given a random rotation with friction (a "shattering" effect)
Undo:
1. friction is max (so they stop spinning)
2. recompute neighbors (spinning and phase 4 messes up their order)
3. Every dot gradually extends the lines to their neighbors

// phase 4 (disconnected edges point at mouse)
*/