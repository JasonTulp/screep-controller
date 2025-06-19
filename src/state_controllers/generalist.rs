use crate::screep_states::*;
use crate::utils::prelude::*;
use crate::{info, utils};
use log::warn;
use screeps::{constants::ResourceType, enums::StructureObject, find, objects::Creep, prelude::*, Part, Room};

use super::{Specialisation, StateController};

/// Generalist State Controller for managing a sawdcreep that performs a variety of tasks
pub struct SCGeneralist {
    pub current_state: Box<dyn ScreepState>,
    pub creep_memory: CreepMemory,
}

impl SCGeneralist {
    pub fn new() -> Self {
        SCGeneralist {
            current_state: Box::new(IdleState {}),
            creep_memory: CreepMemory::new(Specialisation::Generalist),
        }
    }
}

impl StateController for SCGeneralist {
    fn get_name(&self) -> &'static str {
        Specialisation::Generalist.into()
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
        // TODO
        // if sc_manager.get_specialty_count(StateName::Build.into()) < 2 &&
        //     sc_manager.get_specialty_count(StateName::Upgrade.into()) > 0 {
        //     if let Some(site) = find_nearest_construction_site(creep, &room) {
        //         return Box::new(BuildState::new(site.clone()));
        //     }
        // }

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

    fn get_best_worker_body(&self, room: &Room) -> Vec<Part> {
        let mut base_body = vec![Part::Move, Part::Move, Part::Carry, Part::Work];
        let energy_available: u32 = utils::get_total_upgrade_energy(room);
        // info!("Total available: {}", energy_available);
        let mut cost = base_body.iter().map(|p| p.cost()).sum::<u32>();
        while cost < energy_available {
            if cost + Part::Work.cost() <= energy_available {
                base_body.push(Part::Work);
                cost += Part::Work.cost();
            }

            if cost + Part::Move.cost() <= energy_available {
                base_body.push(Part::Move);
                cost += Part::Move.cost();
            }

            if cost + Part::Carry.cost() <= energy_available {
                base_body.push(Part::Carry);
                cost += Part::Carry.cost();
            }
        }

        base_body
    }

    fn get_memory(&self) -> CreepMemory {
        self.creep_memory.clone()
    }
}
