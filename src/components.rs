use rltk::RGB;
use serde::{Deserialize, Serialize};
use specs::error::NoError;
use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs_derive::*;

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
    pub radius: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Confusion {
    pub turns: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}

#[derive(Clone, Component, ConvertSaveload)]
pub struct DefenseBonus {
    pub defense: i32,
}

#[derive(Clone, Copy, Deserialize, PartialEq, Serialize)]
pub enum EquipmentSlot {
    Melee,
    Shield,
}

#[derive(Clone, Component, Deserialize, Serialize)]
pub struct Equippable {
    pub slot: EquipmentSlot,
}

#[derive(Clone, Component, ConvertSaveload)]
pub struct Equipped {
    pub owner: Entity,
    pub slot: EquipmentSlot,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct InBackpack {
    pub owner: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage {
    pub damage: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Clone, Component, ConvertSaveload)]
pub struct MeleePowerBonus {
    pub power: i32,
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Monster {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Name {
    pub name: String,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Player {}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount: i32,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
    pub range: i32,
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}
// Serialization helper code. We need to implement ConvertSaveload for each type that contains an
// Entity.
pub struct SerializeMe;

// Special component that exists to help serialize the game data
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map: super::map::Map,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

impl SufferDamage {
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage {
                amount: vec![amount],
            };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToDropItem {
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToPickupItem {
    pub collected_by: Entity,
    pub item: Entity,
}

#[derive(Clone, Component, ConvertSaveload, Debug)]
pub struct WantsToRemoveItem {
    pub item: Entity,
}

#[derive(Component, Debug, ConvertSaveload)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target: Option<rltk::Point>,
}
