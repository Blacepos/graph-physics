use std::{cmp::Ordering, collections::BinaryHeap};

use bevy::{prelude::*, utils::{HashMap, HashSet}};
use ordered_float::NotNan;

/// How close two dots have to be to be considered neighbors when using distance
const COMPUTE_NEIGHBORS_MAX_DISTANCE: f32 = 80.0;
/// The number of connections to compute for each dot when using K-nearest neighbors
const COMPUTE_NEIGHBORS_K_NEAREST: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComputeNeighborsMethod {
    Distance,
    KNearest,
}

#[derive(Resource, Debug)]
pub struct GraphSpawnConfig {
    pub compute_neighbors_method: ComputeNeighborsMethod,
    pub max_distance: f32,
    pub k_nearest: usize,
}

impl Default for GraphSpawnConfig {
    fn default() -> Self {
        GraphSpawnConfig {
            compute_neighbors_method: ComputeNeighborsMethod::KNearest,
            max_distance: COMPUTE_NEIGHBORS_MAX_DISTANCE,
            k_nearest: COMPUTE_NEIGHBORS_K_NEAREST,
        }
    }
}

/// An entity participating in the interactive animation
#[derive(Component)]
pub struct Dot;

/// An entity which is logically connected to other entities
#[derive(Component)]
pub struct Neighbors {
    pub neighbors: Vec<Entity>
}

/// An entity which is logically connected to a single other entity.
/// NOTE: The assumption is that if A's partner is B, B's partner is A
#[derive(Component)]
pub struct Partner {
    pub partner: Option<Entity>
}

#[derive(PartialEq, Eq)]
struct MinHeapEntry {
    reverse_cmp_distance_sq: NotNan<f32>,
    eid1: Entity,
    eid2: Entity,
}

impl Ord for MinHeapEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.reverse_cmp_distance_sq.partial_cmp(&self.reverse_cmp_distance_sq).unwrap()
    }
}

impl PartialOrd for MinHeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn compute_neighbors(
    q: Query<(&Transform, &mut Neighbors, Entity), With<Dot>>,
    graph_spawn_config: Res<GraphSpawnConfig>,
) {
    match graph_spawn_config.compute_neighbors_method {
        ComputeNeighborsMethod::KNearest => compute_neighbors_by_k_nearest(q, graph_spawn_config),
        ComputeNeighborsMethod::Distance => compute_neighbors_by_distance(q, graph_spawn_config),
    }
}

/// Makes connections between nodes based the K nearest neighbors
pub fn compute_neighbors_by_k_nearest(
    mut q: Query<(&Transform, &mut Neighbors, Entity), With<Dot>>,
    graph_spawn_config: Res<GraphSpawnConfig>
) {
    debug!("empty: {}", q.is_empty());

    // remove all existing neighbors
    for (_, mut neighbors, _) in q.iter_mut() {
        neighbors.neighbors.clear();
    }

    let mut knn_sets: HashMap<Entity, Vec<Entity>> = HashMap::new();

    // compute new neighbors
    for (transform, _, eid) in q.iter() {
        let mut min_heap: BinaryHeap<MinHeapEntry> = BinaryHeap::new();

        let other_nodes = q.iter().filter(|(_, _, potential_neighbor_eid)| *potential_neighbor_eid != eid);
        for (potential_neighbor, _, potential_neighbor_eid) in other_nodes {
            let delta = transform.translation - potential_neighbor.translation;
            let distance_sq = delta.length_squared();

            min_heap.push(MinHeapEntry {
                reverse_cmp_distance_sq: NotNan::new(distance_sq).unwrap(),
                eid1: eid,
                eid2: potential_neighbor_eid,
            });
        }
        
        let k_nearest: Vec<Entity> = min_heap.iter().take(graph_spawn_config.k_nearest).map(|entry| entry.eid2).collect();

        knn_sets.insert(eid, k_nearest);
    }

    for (eid, k_nearest) in knn_sets {
        let mut current_neighbors = q.get_mut(eid).unwrap().1;
        current_neighbors.neighbors.extend_from_slice(&k_nearest);
    }
}

/// Makes connections between nodes based on distance
pub fn compute_neighbors_by_distance(
    mut q: Query<(&Transform, &mut Neighbors, Entity), With<Dot>>,
    graph_spawn_config: Res<GraphSpawnConfig>
) {
    debug!("empty: {}", q.is_empty());

    // remove all existing neighbors
    for (_, mut neighbors, _) in q.iter_mut() {
        neighbors.neighbors.clear();
    }

    // compute new neighbors
    let mut iter = q.iter_combinations_mut();

    while let Some([(current, mut current_neighbors, _), (potential_neighbor, _, potential_neighbor_eid)]) = iter.fetch_next() {
        let delta = current.translation - potential_neighbor.translation;
        let distance_sq = delta.length_squared();

        // add neighbor if it is close enough
        if distance_sq < graph_spawn_config.max_distance.powi(2) {
            current_neighbors.neighbors.push(potential_neighbor_eid)
        }
    }
}

/// Updates every Dot's `Partner` component such that there is no Dot with 2 links (either incoming
/// or outgoing) i.e., pick first neighbor without a partner.
/// This requires being able check a neighbor's `Partner` component.
pub fn compute_disjoint_pairs(
    mut q: Query<(&Neighbors, &mut Partner, Entity), With<Dot>>,
) {
    // remove all existing partners
    for (_, mut partner, _) in q.iter_mut() {
        partner.partner = None;
    }

    let mut new_partners = Vec::new();
    let mut taken = HashSet::new();

    // compute new partners
    for (neighbors, _, eid) in q.iter() {
        // if we have a partner, we skip checking
        if !taken.contains(&eid) {
            for neighbor_eid in neighbors.neighbors.iter().cloned() {      
                // if this neighbor also doesn't have a partner, we make it our partner
                if !taken.contains(&neighbor_eid) {
                    new_partners.push((eid, neighbor_eid));
                    taken.insert(eid);
                    taken.insert(neighbor_eid);
                    break;
                }
            }
        }
    }

    for (eid, neighbor_eid) in new_partners {
        let [(_, mut partner, _), (_, mut neighbors_partner, _)] = q.get_many_mut([eid, neighbor_eid]).unwrap();
    
        partner.partner = Some(neighbor_eid);
        neighbors_partner.partner = Some(eid);
    }
}
