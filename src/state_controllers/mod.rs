// Contains core State Controller logic for managing Screep states
use crate::screep_states::*;
use crate::{info};
use log::warn;
use screeps::{
    constants::ResourceType,
    enums::StructureObject,
    find,
    objects::Creep,
    prelude::*,
};
use std::collections::HashMap;
use crate::utils::prelude::*;

// pub trait StateController {
//     
// }

pub struct StateController {
    pub upgrade_creeps: u8,
    pub build_creeps: u8,
}

impl StateController {
    pub fn new() -> Self {
        StateController {
            upgrade_creeps: 0,
            build_creeps: 0,
        }
    }

    /// Run a tick for the given creep and update its state
    pub fn run_tick(
        &mut self,
        creep: &Creep,
        creep_states: &mut HashMap<String, Box<dyn ScreepState>>,
    ) {
        let name = creep.name();
        if let Some(state) = creep_states.get_mut(&name) {
            match state.tick(creep) {
                TickResult::Continue => {
                    // Continue running the current state
                    return;
                }
                TickResult::ChangeState(new_state) => {
                    // Exit the current state
                    state.on_exit(self);
                    new_state.on_start(creep, self);
                    new_state.log_state(creep);
                    // Insert the new state
                    creep_states.insert(name.clone(), new_state);
                }
                TickResult::Exit => {
                    // Exit the current state and remove it from the map
                    state.on_exit(self);
                    let new_state: Box<dyn ScreepState> = self.choose_next_state(creep);
                    new_state.on_start(creep, self);
                    new_state.log_state(creep);
                    creep_states.insert(name.clone(), new_state);
                }
            }
        } else {
            // If no state exists, we can initialize a default state
            let initial_state: Box<dyn ScreepState> = self.choose_next_state(creep);
            initial_state.on_start(creep, self);
            initial_state.log_state(creep);
            creep_states.insert(name, initial_state);
        }
    }

    /// Choose the next state based on the current needs of the room
    pub fn choose_next_state(&mut self, creep: &Creep) -> Box<dyn ScreepState> {
        let room = creep.room().expect("couldn't resolve creep room");
        let energy = creep.store().get_used_capacity(Some(ResourceType::Energy));
        if energy == 0 {
            // Attempt to find some sources to harvest
            if let Some(source) = find_nearest_object(creep, &room, find::SOURCES_ACTIVE) {
                info!("Starting Harvest State for creep {}", creep.name());
                return Box::new(HarvestState::new(source));
            } else {
                warn!("No sources found for creep {}", creep.name());
                return Box::new(IdleState {});
            }
        }

        // Check if the base needs energy
        let upgrade_energy = get_total_upgrade_energy(&room);
        let energy_available = room.energy_available();
        if energy_available < upgrade_energy {
            // Find a structure to feed energy to
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureSpawn(spawn) = structure {
                    if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        info!(
                            "Starting Feed spawn Structure State for creep {}",
                            creep.name()
                        );
                        return Box::new(
                            FeedStructureState::<screeps::objects::StructureSpawn>::new(spawn.id()),
                        );
                    }
                } else if let StructureObject::StructureExtension(extension) = structure {
                    if extension
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        > 0
                    {
                        info!(
                            "Starting Feed Extension Structure State for creep {}",
                            creep.name()
                        );
                        return Box::new(
                            FeedStructureState::<screeps::objects::StructureExtension>::new(
                                extension.id(),
                            ),
                        );
                    }
                }
            }
        }

        // limit build creeps to 2, only build if we have an upgrade creep
        if self.build_creeps < 2 && self.upgrade_creeps > 0 {
            if let Some(site) = find_nearest_construction_site(creep, &room) {
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
}
