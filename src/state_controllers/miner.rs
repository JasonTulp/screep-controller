use crate::screep_states::*;
use crate::{info, utils};
use log::warn;
use screeps::{constants::ResourceType, enums::StructureObject, find, objects::Creep, prelude::*, Part, Room};
use crate::utils::find_object_at_index;
use super::{Specialisation, StateController};

/// Miner State Controller for mining energy and dumping it into nearby storage
pub struct SCMiner {
    pub current_state: Box<dyn ScreepState>,
    // index of resource to mine in room
    pub source_index: Option<u8>,
}

impl SCMiner {
    pub fn new() -> Self {
        SCMiner {
            current_state: Box::new(IdleState {}),
            source_index: None
        }
    }

    // Get the source index either from the cached state, the memory or finding a new one
    fn get_source_index(&mut self, creep: &Creep) -> Option<u8> {
        // If source_index is already set, return it
        if let Some(index) = self.source_index {
            return Some(index);
        }

        // Try to get the index from memory
        if let Some(index) = self.get_index_from_memory(creep) {
            return Some(index);
        }

        // If not found in memory, find the source index
        let room = creep.room().expect("couldn't resolve creep room");
        self.find_source_index(&room, creep)
    }

    // Get index if stored in memory
    fn get_index_from_memory(&mut self, creep: &Creep) -> Option<u8> {
        // Get the source index from memory if it exists
        let memory: CreepMemory = creep.memory().into();
        if let Some(data) = memory.additional_data() {
            if let Ok(index) = data.parse::<u8>() {
                self.source_index = Some(index);
            } else {
                warn!("Invalid source index in memory for creep {}", creep.name());
                self.source_index = None;
            }
        } else {
            self.source_index = None;
        }
        self.source_index
    }

    // Find the resource index by getting the source with the least number of miners
    fn find_source_index(&mut self, room: &Room, creep: &Creep) -> Option<u8> {
        // get room sources
        let sources: Vec<u8> = room.find(find::SOURCES_ACTIVE, None)
            .into_iter().enumerate().map(|( i, _)| i as u8).collect();

        // Count miners on each source
        let mut source_counts: Vec<(u8, usize)> = sources.iter().map(|&source_index| (source_index, 0)).collect();
        
        // Count existing miners on each source
        room.find(find::CREEPS, None).iter().for_each(|creep| {
            let memory = CreepMemory::from(creep.memory());
            if memory.specialisation() == &Specialisation::Miner {
                if let Some(data) = memory.additional_data() {
                    if let Ok(source_index) = data.parse::<u8>() {
                        if let Some((_, count)) = source_counts.iter_mut().find(|(idx, _)| *idx == source_index) {
                            *count += 1;
                        }
                    }
                }
            }
        });
        
        // Find the source with the least number of miners
        let new_index = source_counts
            .iter()
            .min_by_key(|(_, count)| *count)
            .map(|(source_index, _)| *source_index);
        self.source_index = new_index;

        // Set memory of the miner to the new source index
        if let Some(new_index) = new_index {
            let mut memory: CreepMemory = creep.memory().into();
            memory.set_additional_data(new_index.to_string());
            creep.set_memory(&memory.into());
        }
        new_index
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

        // Use the source_index if it exists, otherwise find and set it
        let Some(source_index) = self.get_source_index(creep) else {
            return Box::new(IdleState {});
        };

        if energy == 0 {
            // Find the resource
            if let Some(source) = find_object_at_index(&room, source_index, find::SOURCES_ACTIVE) {
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
                if let StructureObject::StructureContainer(container) = structure {
                    if container.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        return Box::new(
                            FeedStructureState::<screeps::objects::StructureContainer>::new(container.id()),
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

    /// Get the best worker body for this state controller
    fn get_best_worker_body(&self, room: &Room) -> Vec<Part> {
        let mut base_body = vec![Part::Move, Part::Carry, Part::Work];
        let target_body = vec![
            Part::Move,
            Part::Carry,
            Part::Work,
            Part::Work,
            Part::Move,
            Part::Work,
            Part::Work,
            Part::Move,
            Part::Work,
            Part::Work,
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
