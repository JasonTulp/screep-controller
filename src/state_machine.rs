use std::collections::HashMap;
use log::warn;
use screeps::{
    constants::{Part, ResourceType},
    enums::StructureObject,
    find, game,
    local::ObjectId,
    objects::{Creep, Room, Source, ConstructionSite, StructureController},
    prelude::*,
};
use crate::info;
use crate::harvest_state::HarvestState;
use crate::upgrade_state::UpgradeState;
use crate::build_state::BuildState;
use crate::feed_structure_state::FeedStructureState;
use crate::get_total_upgrade_energy;

// Result from a tick
pub enum TickResult {
    // Keep the state as-is
    Continue,
    // Change to a specific state
    ChangeState(Box<dyn ScreepState>),
    // exit and choose a state based on current needs
    Exit,
}

// What state is this screep in
pub trait ScreepState {
    fn on_start(&self, creep: &Creep, state_controller: &mut StateController);
    fn tick(&mut self, creep: &Creep) -> TickResult;
    fn on_exit(&self, state_controller: &mut StateController);
}

pub struct StateController {
    pub upgrade_creeps: u8,
    pub build_creeps: u8,
    pub  harvest_creeps: u8,
}

impl StateController {
    pub fn new() -> Self {
        StateController {
            upgrade_creeps: 0,
            build_creeps: 0,
            harvest_creeps: 0,
        }
    }

    /// Run a tick for the given creep and update its state
    pub fn run_tick(&mut self, creep: &Creep, creep_states: &mut HashMap<String, Box<dyn ScreepState>>) {
        let name = creep.name();
        if let Some(state) = creep_states.get_mut(&name) {
            match state.tick(creep) {
                TickResult::Continue => {
                    // Continue running the current state
                    return;
                },
                TickResult::ChangeState(new_state) => {
                    // Exit the current state
                    state.on_exit(self);
                    new_state.on_start(creep, self);
                    // Insert the new state
                    creep_states.insert(name.clone(), new_state);
                },
                TickResult::Exit => {
                    // Exit the current state and remove it from the map
                    state.on_exit(self);
                    let new_state: Box<dyn ScreepState> = self.choose_next_state(creep);
                    new_state.on_start(creep, self);
                    creep_states.insert(name.clone(), new_state);
                },
            }
        } else {
            // If no state exists, we can initialize a default state
            let initial_state: Box<dyn ScreepState> = self.choose_next_state(creep);
            initial_state.on_start(creep, self);
            creep_states.insert(name, initial_state);
        }
    }

    /// Choose the next state based on the current needs of the room
    pub fn choose_next_state(
        &mut self,
        creep: &Creep
    ) -> Box<dyn ScreepState> {
        let room = creep.room().expect("couldn't resolve creep room");
        let energy = creep.store().get_used_capacity(Some(ResourceType::Energy));
        if energy == 0 {
            // Attempt to find some sources to harvest
            if let Some(source) = self.find_nearest_source(creep, &room) {
                info!("Starting Harvest State for creep {}", creep.name());
                return Box::new(HarvestState::new(source));
            } else {
                warn!("No sources found for creep {}", creep.name());
                return Box::new(IdleState {});
            }
        }

        // check how many creeps we have, if below 5. prioritise spawning more by fueling spawner
        let creep_count = game::creeps().values().count();
        let upgrade_energy = get_total_upgrade_energy(&room);
        let energy_available = room.energy_available();
        if energy_available < upgrade_energy {
            // Find a structure to feed energy to
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureSpawn(spawn) = structure {
                    if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        info!("Starting Feed spawn Structure State for creep {}", creep.name());
                        return Box::new(FeedStructureState::<screeps::objects::StructureSpawn>::new(spawn.id()));
                    }
                }
                else if let StructureObject::StructureExtension(extension) = structure {
                    if extension.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        info!("Starting Feed Extension Structure State for creep {}", creep.name());
                        return Box::new(FeedStructureState::<screeps::objects::StructureExtension>::new(extension.id()));
                    }
                }
            }
        }

        // limit build creeps to 2, only build if we have an upgrade creep
        if self.build_creeps < 2 && self.upgrade_creeps > 0 {
            if let Some(site) = self.find_nearest_construction_site(creep, &room) {
                return Box::new(BuildState::new(site.clone()));
            }
        }

        // Check if we have energy, if we do, upgrade controller
        if energy > 0 {
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureController(controller) = structure {
                    info!("Starting Upgrade State for creep {}", creep.name());
                    return Box::new(UpgradeState::new(controller.id()));
                }
            }
        }


        // return idle state if no other states are compatible
        info!("Starting Idle State for creep {}", creep.name());
        Box::new(IdleState {})
    }

    // Gets the nearest source based on distance from the creep
    fn find_nearest_source(
        &mut self,
        creep: &Creep,
        room: &Room
    ) -> Option<ObjectId<Source>> {
        let sources = room.find(find::SOURCES_ACTIVE, None);
        if sources.is_empty() {
            return None;
        }
        // Find the nearest source
        let nearest_source = sources.iter().min_by_key(|source| {
            creep.pos().get_range_to(source.pos())
        })?;

        Some(nearest_source.id())
    }

    fn find_nearest_construction_site(
        &mut self,
        creep: &Creep,
        room: &Room
    ) -> Option<ConstructionSite> {
        let sites = room.find(find::CONSTRUCTION_SITES, None);
        if sites.is_empty() {
            return None;
        }
        // Find the nearest construction site
        let nearest_site = sites.iter().min_by_key(|site| {
            creep.pos().get_range_to(site.pos())
        })?;

        Some(nearest_site.clone())
    }
}


/// Idle state can be used as a fallback when no other state is applicable
/// This state will do nothing and constantly search for a better state on each tick
pub struct IdleState;
impl ScreepState for IdleState {
    fn on_start(&self, creep: &Creep, state_controller: &mut StateController) {
        let _ = creep.say("💤", false);
    }
    fn tick(&mut self, _creep: &Creep) -> TickResult {
        // Do nothing, just idle until new state can be chosen
        TickResult::Exit
    }
    fn on_exit(&self, state_controller: &mut StateController) {
        return;
    }
}
