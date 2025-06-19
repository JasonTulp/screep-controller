use crate::screep_states::*;
use crate::utils::prelude::*;
use crate::{info, utils};
use log::warn;
use screeps::{constants::ResourceType, enums::StructureObject, find, objects::Creep, prelude::*, Part, Room};
use crate::utils::find_object_at_index;
use super::{Specialisation, StateController};

/// Miner State Controller for mining energy and dumping it into nearby storage
pub struct SCMiner {
    pub current_state: Box<dyn ScreepState>,
    pub creep_memory: CreepMemory,
    // index of resource to mine in room
    pub resource_index: u8,
}

impl SCMiner {
    pub fn new() -> Self {
        // TODO get resource index from external
        SCMiner {
            current_state: Box::new(IdleState {}),
            creep_memory: CreepMemory::new(Specialisation::Miner),
            resource_index: 0
        }
    }
}

impl StateController for SCMiner {
    fn get_name(&self) -> &'static str {
        Specialisation::Miner.into()
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
            // Find the resource
            if let Some(source) = find_object_at_index(&room, self.resource_index, find::SOURCES_ACTIVE) {
                return Box::new(HarvestState::new(source));
            } else {
                warn!("No sources found for creep {}", creep.name());
                return Box::new(IdleState {});
            }
        }

        // Check if we have energy, if we do, upgrade controller
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
                    info!("Starting Upgrade State for creep {}", creep.name());
                    return Box::new(UpgradeState::new(controller.id()));
                }
            }
        }

        // return idle state if no other states are compatible
        Box::new(IdleState {})
    }

    fn get_best_worker_body(&self, room: &Room) -> Vec<Part> {
        let mut base_body = vec![Part::Move, Part::Move, Part::Carry, Part::Work];
        let energy_available: u32 = utils::get_total_upgrade_energy(room);
        let mut cost = base_body.iter().map(|p| p.cost()).sum::<u32>();
        while cost < energy_available {
            if cost + Part::Work.cost() <= energy_available {
                base_body.push(Part::Work);
                cost += Part::Work.cost();
            }
            
            // Two work for every 1 move
            if cost + Part::Work.cost() <= energy_available {
                base_body.push(Part::Work);
                cost += Part::Work.cost();
            }

            if cost + Part::Move.cost() <= energy_available {
                base_body.push(Part::Move);
                cost += Part::Move.cost();
            }
        }

        base_body
    }

    fn get_memory(&self) -> CreepMemory {
        self.creep_memory.clone()
    }
}
