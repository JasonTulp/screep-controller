use crate::screep_states::*;
use crate::utils;
use crate::utils::prelude::*;
use log::warn;
use screeps::{constants::ResourceType, enums::StructureObject, find, objects::Creep, prelude::*, ObjectId, Part, Room};

use super::{Specialisation, StateController};

/// Builder State Controller for managing a dedicated upgrader creep
pub struct SCUpgrader {
    pub current_state: Box<dyn ScreepState>,
}

impl SCUpgrader {
    pub fn new() -> Self {
        SCUpgrader {
            current_state: Box::new(IdleState {}),
        }
    }
}

impl StateController for SCUpgrader {
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
                // Otherwise, attempt to find some sources to harvest
                if let Some(source) = find_nearest_object(&creep.pos(), &room, find::SOURCES_ACTIVE) {
                    return Box::new(HarvestState::new(source));
                } else {
                    warn!("No sources found for creep {}", creep.name());
                    return Box::new(IdleState {});
                }
            }
        }

        // upgrade controller if nothing to build
        for structure in room.find(find::STRUCTURES, None).iter() {
            if let StructureObject::StructureController(controller) = structure {
                return Box::new(UpgradeState::new(controller.id()));
            }
        }

        // return idle state if no other states are compatible
        Box::new(IdleState {})
    }

    fn get_best_worker_body(&self, room: &Room) -> Vec<Part> {
        let mut base_body = vec![];
        let blueprint = vec![
            Part::Move,
            Part::Carry,
            Part::Work,
            Part::Work,
            Part::Move,
            Part::Move,
        ];
        let blueprint_cost = blueprint.iter().map(|p: &Part| p.cost()).sum::<u32>();
        let energy_available: u32 = utils::get_total_upgrade_energy(room);
        let mut cost = base_body.iter().map(|p: &Part| p.cost()).sum::<u32>();

        // keep adding parts from blueprint until we reach the energy limit
        while cost + blueprint_cost <= energy_available {
            for part in blueprint.iter() {
                base_body.push(*part);
                cost += part.cost();
            }
        }

        base_body
    }
}
