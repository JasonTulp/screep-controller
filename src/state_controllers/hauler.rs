use super::{Specialisation, StateController};
use crate::screep_states::*;
use crate::utils::prelude::*;
use log::warn;
use screeps::{constants::ResourceType, enums::StructureObject, find, objects::Creep, prelude::*, Part, StructureTower};

/// Hauler State Controller for getting energy from containers and moving to the Storage
pub struct SCHauler {
    pub current_state: Box<dyn ScreepState>,
}

impl SCHauler {
    pub fn new() -> Self {
        SCHauler {
            current_state: Box::new(IdleState {}),
        }
    }
}

impl StateController for SCHauler {
    fn get_name(&self) -> &'static str {
        Specialisation::Hauler.into()
    }

    fn get_blueprint(&self) -> Vec<Part> {
        vec![
            Part::Move,
            Part::Carry,
        ]
    }

    fn current_state(&self) -> &Box<dyn ScreepState> {
        &self.current_state
    }

    fn set_current_state(&mut self, state: Box<dyn ScreepState>) {
        self.current_state = state;
    }

    fn choose_next_state(&mut self, creep: &Creep) -> Box<dyn ScreepState> {
        let room = creep.room().expect("couldn't resolve creep room");
        let energy = creep.store().get_used_capacity(Some(ResourceType::Energy));
        if energy == 0 {
            // Find the container with the most energy to drain
            if let Some(container_id) = find_container_with_most_energy(&room) {
                return Box::new(WithdrawState::new(container_id));
            } else {
                warn!("No containers found for creep {}", creep.name());
                return Box::new(IdleState {});
            }
        }

        // Attempt to find a storage structure to feed energy to
        for structure in room.find(find::STRUCTURES, None).iter() {
            if let StructureObject::StructureStorage(storage) = structure {
                if storage
                    .store()
                    .get_free_capacity(Some(ResourceType::Energy))
                    > 0
                {
                    return Box::new(
                        FeedStructureState::<screeps::objects::StructureStorage>::new(
                            storage.id(),
                        ),
                    );
                }
            }
        }

        // Check if the base needs energy
        if let Some(state) = find_base_structure_needing_energy(&room) {
            return state;
        }

        // Check if we have towers that need energy
        if let Some(tower) = find_closest_tower_needing_energy(&room, creep) {
            return Box::new(FeedStructureState::<StructureTower>::new(tower));
        }

        // Try fill up from container because we have nothing better to do...
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) > 0
        {
            // Find the container with the most energy to drain
            let best_container = find_container_with_most_energy(&room);
            // If we found a container with energy, harvest from it
            if let Some(container_id) = best_container {
                return Box::new(WithdrawState::new(container_id));
            } else {
                warn!("No containers found for creep {}", creep.name());
                return Box::new(IdleState {});
            }
        }

        // return idle state if no other states are compatible
        Box::new(IdleState {})
    }
}
