use bevy::prelude::*;

use std::cmp::Ordering;

use itertools::Itertools;

use crate::GameState;

pub struct ZSortPlugin;

// NOTE: This module/plugin is very small. It may make sense to group this functionality somewhere
// else in the future, however it was unclear where to put it at time of writing.
impl Plugin for ZSortPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, z_sort.run_if(in_state(GameState::InGame)));
    }
}

/// The minimum Z value for transforms on the "Player" layer, i.e. players and bombs.
pub const PLAYER_Z: f32 = 10.0;

#[derive(Component)]
pub struct ZSort(pub f32 /*the min Z value*/);

/// Sort the players on the Z-axis based on their position on the Y-axis.
//
// NOTE: Consider benchmarking against a strategy of linear interpolation wrt. the min/max expected
// Y values.
fn z_sort(mut to_sort: Query<(&mut Transform, &ZSort)>) {
    // Iterates over `ZSort`s and groups them by layer, before sorting each group by Y-pos and
    // applying an offset to each element.

    let to_sort = &to_sort.iter_mut().group_by(|(_, zsort)| zsort.0);

    for (z, group) in to_sort {
        let mut to_sort = group.map(|(t, _)| t).collect::<Vec<_>>();
        // Bevy uses a stable sort when rendering entities, so we can take advantage of an unstable one
        // here.
        to_sort.sort_unstable_by(|t1, t2| {
            t2.translation
                .y
                .partial_cmp(&t1.translation.y)
                .unwrap_or(Ordering::Equal)
        });

        for (i, transform) in to_sort.iter_mut().enumerate() {
            transform.translation.z = z + i as f32;
        }
    }
}
