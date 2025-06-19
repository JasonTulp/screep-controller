use super::{Specialisation, StateController};
use crate::screep_states::*;
use crate::utils;
use log::{info, warn};
use screeps::{
    constants::ResourceType, enums::StructureObject, find, objects::Creep, prelude::*, ObjectId,
    Part, Room,
};
use crate::utils::{find_nearest_construction_site, get_total_upgrade_energy};

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

    fn find_container(&self, room: &Room) -> Option<ObjectId<screeps::objects::StructureContainer>> {
        // Find the container with the most energy to drain
        let mut best_container: Option<ObjectId<screeps::objects::StructureContainer>> = None;
        let mut max_energy = 0;
        for structure in room.find(find::STRUCTURES, None).iter() {
            if let StructureObject::StructureContainer(container) = structure {
                if container
                    .store()
                    .get_used_capacity(Some(ResourceType::Energy))
                    > 0
                {
                    let energy_in_container = container
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy));
                    if energy_in_container > max_energy {
                        max_energy = energy_in_container;
                        best_container = Some(container.id());
                    }
                }
            }
        }

        best_container
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
            let best_container = self.find_container(&room);

            // If we found a container with energy, harvest from it
            if let Some(container_id) = best_container {
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
        let upgrade_energy = get_total_upgrade_energy(&room);
        let energy_available = room.energy_available();
        if energy_available < upgrade_energy {
            // Find a structure to feed energy to
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureSpawn(spawn) = structure {
                    if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
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
                        return Box::new(
                            FeedStructureState::<screeps::objects::StructureExtension>::new(
                                extension.id(),
                            ),
                        );
                    }
                }
            }
        }

        // Check if we have towers that need energy
        for structure in room.find(find::STRUCTURES, None).iter() {
            if let StructureObject::StructureTower(tower) = structure {
                if tower.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                    return Box::new(FeedStructureState::<screeps::objects::StructureTower>::new(
                        tower.id(),
                    ));
                }
            }
        }

        // Try fill up from container because we have nothing better to do...
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) > 0
        {
            // Find the container with the most energy to drain
            let best_container = self.find_container(&room);
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
        let mut cost = base_body.iter().map(|p: &Part| p.cost()).sum::<u32>();
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
