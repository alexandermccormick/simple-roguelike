use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::HashMap;

use crate::random_table::RandomTable;
use crate::{Rect, MAPWIDTH};

use super::items::{confusion_scroll, fireball_scroll, health_potion, magic_missile_scroll};
use super::monsters::{goblin, orc};

const MAX_MONSTERS: i32 = 4;

/// Spawns a room full of stuff!
#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;

        match spawn.1.as_ref() {
            "Confusion Scroll" => confusion_scroll(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Goblin" => goblin(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            _ => {}
        }
    }
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Confusion Scroll", 2 + map_depth)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Goblin", 10)
        .add("Health Potion", 7)
        .add("Magic Missile Scroll", 4)
        .add("orc", 1 + map_depth)
}
