use bevy::ecs::{schedule::NextState, system::ResMut};
use bevy_egui::{egui, EguiContexts};

use crate::{graph::{ComputeNeighborsMethod, GraphSpawnConfig}, phases::{Phases, SpawnMethod}, physics::PhysicsConfig};

pub fn ui_tweak_panel(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<Phases>>,
    mut physics_config: ResMut<PhysicsConfig>,
    mut spawn_method: ResMut<SpawnMethod>,
    mut graph_spawn_config: ResMut<GraphSpawnConfig>,
) {
    egui::Window::new("Tweaks").show(contexts.ctx_mut(), |ui| {
        egui::ComboBox::from_label("Spawn Method")
            .selected_text(format!("{:?}", spawn_method.as_ref()))
            .show_ui(ui, |ui| {
                ui.selectable_value(spawn_method.as_mut(), SpawnMethod::Random, "Random");
                ui.selectable_value(spawn_method.as_mut(), SpawnMethod::Grid, "Grid");
            });
        
        egui::ComboBox::from_label("Compute Neighbors Method")
            .selected_text(format!("{:?}", graph_spawn_config.compute_neighbors_method))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut graph_spawn_config.compute_neighbors_method, ComputeNeighborsMethod::KNearest, "KNearest");
                ui.selectable_value(&mut graph_spawn_config.compute_neighbors_method, ComputeNeighborsMethod::Distance, "Distance");
            });

        match graph_spawn_config.compute_neighbors_method {
            ComputeNeighborsMethod::KNearest => {
                ui.add(egui::Slider::new(&mut graph_spawn_config.k_nearest, 1..=10).text("K Nearest"));
            }
            ComputeNeighborsMethod::Distance => {
                ui.add(egui::Slider::new(&mut graph_spawn_config.max_distance, 0.0..=150.0).text("Max Distance"));
            }
        }

        if ui.button("Reset").clicked() {
            next_state.set(Phases::Init);
            println!("Reset phase");
        }

        ui.separator();

        ui.add(egui::Slider::new(&mut physics_config.repel_strength, 0.0..=2000.0).text("Repel Strength"));
        ui.add(egui::Slider::new(&mut physics_config.spring_coefficient, 0.0..=0.1).text("Spring Coefficient"));
        ui.add(egui::Slider::new(&mut physics_config.spring_resting_length, 0.0..=200.0).text("Spring Resting Length"));
        ui.add(egui::Slider::new(&mut physics_config.wall_repel_strength, 0.0..=2000.0).text("Wall Repel Strength"));
        ui.add(egui::Slider::new(&mut physics_config.vel_dampening, 0.0..=1.0).text("Velocity Dampening"));
        ui.add(egui::Slider::new(&mut physics_config.vel_cap, 0.0..=200.0).text("Velocity Cap"));
        ui.add(egui::Slider::new(&mut physics_config.acc_dampening, 0.0..=1.0).text("Acceleration Dampening"));
        ui.add(egui::Slider::new(&mut physics_config.acc_cap, 0.0..=200.0).text("Acceleration Cap"));

    });
}