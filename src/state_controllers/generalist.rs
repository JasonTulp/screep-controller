use crate::screep_states::*;
use crate::utils;
use crate::utils::prelude::*;
use log::warn;
use screeps::{constants::ResourceType, enums::StructureObject, find, objects::Creep, prelude::*, ObjectId, Part, Room};

use super::{Specialisation, StateController};

/// Generalist State Controller for managing a sawdcreep that performs a variety of tasks
pub struct SCGeneralist {
    pub current_state: Box<dyn ScreepState>,
}

impl SCGeneralist {
    pub fn new() -> Self {
        SCGeneralist {
            current_state: Box::new(IdleState {}),
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
            // Find the closest container with energy to drain
            let mut closest_container: Option<ObjectId<screeps::objects::StructureContainer>> = None;
            let mut min_distance = u32::MAX;
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureContainer(container) = structure {
                    if container
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy))
                        > 0
                    {
                        let distance = creep.pos().get_range_to(container.pos());
                        if distance < min_distance {
                            min_distance = distance;
                            closest_container = Some(container.id());
                        }
                    }
                }
            }

            // If we found a container with energy, harvest from it
            if let Some(container_id) = closest_container {
                return Box::new(WithdrawState::new(container_id));
            } else {
                // Attempt to find some sources to harvest
                if let Some(source) = find_nearest_object(&creep.pos(), &room, find::SOURCES_ACTIVE) {
                    return Box::new(HarvestState::new(source));
                } else {
                    warn!("No sources found for creep {}", creep.name());
                    return Box::new(IdleState {});
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

        let build_count = self.count_state_instances(&room, &StateName::Build);
        let upgrade_count = self.count_state_instances(&room, &StateName::Upgrade);
        // limit build creeps to 2, only build if we have an upgrade creep
        if build_count < 2 && upgrade_count > 0 {
            if let Some(site) = find_nearest_construction_site(creep, &room) {
                return Box::new(BuildState::new(site.clone()));
            }
        }

        // upgrade controller
        for structure in room.find(find::STRUCTURES, None).iter() {
            if let StructureObject::StructureController(controller) = structure {
                return Box::new(UpgradeState::new(controller.id()));
            }
        }

        // return idle state if no other states are compatible
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
}
