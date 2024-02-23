use bevy::prelude::*;

use crate::graph::{Dot, Neighbors, Partner};

const DOT_CIRCLE_RADIUS: f32 = 4.0;

/// Render lines based on the values of every Dot's Neighbors component
pub fn render_graph_edges(
    nodes_query: Query<(&Neighbors, &Transform), With<Dot>>,
    mut gizmos: Gizmos
) {
    for (neighbors, transform) in nodes_query.iter() {
        let pos = transform.translation;
        for neighbor_eid in neighbors.neighbors.iter().cloned() {
            let neighbor_pos = nodes_query.get(neighbor_eid).unwrap().1.translation;

            // render the line
            gizmos.line_2d(pos.xy(), neighbor_pos.xy(), Color::WHITE);
        }
    }
}

/// Render a circle for every Dot
pub fn render_dots(
    q: Query<&Transform, With<Dot>>,
    mut gizmos: Gizmos
) {
    for transform in q.iter() {
        gizmos.circle_2d(transform.translation.xy(), DOT_CIRCLE_RADIUS, Color::WHITE).segments(8);
    }
}

/// Render lines based on the values of every Dot's Partner component
pub fn render_partners(
    q: Query<(&Partner, &Transform), With<Dot>>,
    mut gizmos: Gizmos
) {
    for (partner, transform) in q.iter() {
        let pos = transform.translation;
        if let Some(partner_eid) = partner.partner {
            let partner_pos = q.get(partner_eid).unwrap().1.translation;

            // render the line
            gizmos.line_2d(pos.xy(), partner_pos.xy(), Color::WHITE);
        }
    }
}
