use rltk::{GameState, Point, Rltk};
use specs::{prelude::*, saveload::{SimpleMarker, SimpleMarkerAllocator}};

mod components;
mod player;
mod map;
mod rect;
mod visibility_system;
mod monster_ai_system;
mod map_indexing_system;
mod melee_combat_system;
mod damage_system;
mod gui;
mod gamelog;
mod spawner;
mod inventory_system;
mod menu;
mod saveload_system;

pub use components::*;
pub use player::*;
pub use map::*;
pub use rect::*;
use visibility_system::VisibilitySystem;
use monster_ai_system::MonsterAI;
use map_indexing_system::MapIndexingSystem;
use melee_combat_system::MeleeCombatSystem;
use damage_system::{DamageSystem, delete_the_dead};
use inventory_system::{ItemCollectionSystem, UseItemSystem, ItemDropSystem};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity},
    MainMenu { menu_selection: menu::MainMenuSelection },
    SaveGame
}

pub struct State {
    pub ecs: World,
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate = {
            let runstate = self.ecs.fetch::<RunState>();
            *runstate
        };

        ctx.cls();

        match newrunstate {
            RunState::MainMenu { .. } => {}
            _ => {
                draw_map(&self.ecs, ctx);
                {
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let map = self.ecs.fetch::<Map>();

                    let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
                    data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
                    for (pos, render) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] { ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) }
                    }
                    gui::draw_ui(&self.ecs, ctx);
                }
            }
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::MainMenu { .. } => {
                let result = menu::main_menu(self, ctx);
                match result {
                    menu::MainMenuResult::NoSelection { selected } => newrunstate = RunState::MainMenu { menu_selection: selected },
                    menu::MainMenuResult::Selected { selected } => {
                        match selected {
                            menu::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                            menu::MainMenuSelection::LoadGame => {
                                saveload_system::load_game(&mut self.ecs);
                                newrunstate = RunState::AwaitingInput;
                                saveload_system::delete_save();
                            },
                            menu::MainMenuSelection::Quit => { ::std::process::exit(0); }
                        }
                    }
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);

                newrunstate = RunState::MainMenu { menu_selection: menu::MainMenuSelection::LoadGame };
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting { range: is_item_ranged.range, item: item_entity };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item: item_entity, target: None })
                                .expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToDropItem{ item: item_entity }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem { item, target: result.1 })
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }

        delete_the_dead(&mut self.ecs);
    }
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem{};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI{};
        mob.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem{};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem{};
        damage.run_now(&self.ecs);

        let mut pickup = ItemCollectionSystem{};
        pickup.run_now(&self.ecs);

        let mut potions = UseItemSystem{};
        potions.run_now(&self.ecs);

        let mut drop_items = ItemDropSystem{};
        drop_items.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;

    let map = Map::new_map_rooms_and_corridors();
    let (player_x, player_y) = map.rooms[0].center();

    let mut context = RltkBuilder::simple80x50()
        .with_title("Rougelike Tutorial")
        .build()?;
    context.with_post_scanlines(true);

    let mut gs = State {
        ecs: World::new(),
    };


    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Position>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<SimpleMarker<SerializeMe>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(RunState::MainMenu { menu_selection: menu::MainMenuSelection::NewGame });
    gs.ecs.insert(gamelog::GameLog{ entries: vec!["Welcome to Rusty Roguelike".to_string()] });
    gs.ecs.insert(rltk::RandomNumberGenerator::new());

    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);
    gs.ecs.insert(player_entity);

    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);

    rltk::main_loop(context, gs)
}
