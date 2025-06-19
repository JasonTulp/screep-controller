use crate::screep_states::*;
use crate::{info, utils};
use log::warn;
use screeps::{constants::ResourceType, enums::StructureObject, find, objects::Creep, prelude::*, ObjectId, Part, Room};
use crate::utils::find_object_at_index;
use super::{Specialisation, StateController};

/// Hauler State Controller for getting energy from containers and moving to the Storage
pub struct SCHauler {
    pub current_state: Box<dyn ScreepState>,
    // index of resource to mine in room
    pub source_index: Option<u8>,
}

impl SCHauler {
    pub fn new() -> Self {
        SCHauler {
            current_state: Box::new(IdleState {}),
            source_index: None
        }
    }
}

impl StateController for SCHauler {
    fn get_name(&self) -> &'static str {
        Specialisation::Hauler.into()
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
            let mut best_container: Option<ObjectId<screeps::objects::StructureContainer>> = None;
            let mut max_energy = 0;
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureContainer(container) = structure {
                    if container.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        let energy_in_container = container.store().get_used_capacity(Some(ResourceType::Energy));
                        if energy_in_container > max_energy {
                            max_energy = energy_in_container;
                            best_container = Some(container.id());
                        }
                    }
                }
            }

            // If we found a container with energy, harvest from it
            if let Some(container_id) = best_container {
                return Box::new(WithdrawState::new(container_id));
            } else {
                warn!("No containers found for creep {}", creep.name());
                return Box::new(IdleState {});
            }
        }

        // Check if we have energy, deposit into the storage
        if energy > 0 {
            // Attempt to find a storage structure to feed energy to
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureStorage(storage) = structure {
                    if storage.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        return Box::new(
                            FeedStructureState::<screeps::objects::StructureStorage>::new(storage.id()),
                        );
                    }
                }
            }

            // If no storage found, try to upgrade the controller instead
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureController(controller) = structure {
                    return Box::new(UpgradeState::new(controller.id()));
                }
            }
        }

        // return idle state if no other states are compatible
        Box::new(IdleState {})
    }

    // TODO What is the best ratio for carry to move?
    fn get_best_worker_body(&self, room: &Room) -> Vec<Part> {
        let mut base_body = vec![];
        let target_body = vec![
            Part::Move,
            Part::Carry,
            Part::Move,
            Part::Carry,
            Part::Move,
            Part::Carry,
            Part::Carry,
            Part::Move,
            Part::Carry,
            Part::Carry,
            Part::Move,
            Part::Carry,
            Part::Carry,
            Part::Move,
        ];
        let energy_available: u32 = utils::get_total_upgrade_energy(room);
        let mut cost = base_body.iter().map(|p| p.cost()).sum::<u32>();
        // keep adding parts from target until we reach the energy limit
        for part in target_body.iter() {
            if cost + part.cost() <= energy_available {
                base_body.push(*part);
                cost += part.cost();
            } else {
                break;
            }
        }

        // Fill the rest with move (They are only 50 energy each)
        while cost + Part::Move.cost() <= energy_available {
            base_body.push(Part::Move);
            cost += Part::Move.cost();
        }

        base_body
    }
}
