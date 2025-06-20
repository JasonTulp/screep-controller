use super::{Specialisation, StateController};
use crate::screep_states::*;
use crate::utils::prelude::*;
use screeps::{
     objects::Creep, prelude::*, Part, Room,
};
use crate::utils::upgrade_controller;

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
        // Attempt to find energy if we need it
        if let Some(energy_state) = find_energy(&room, creep, EnergyAuthority::All) {
            return energy_state;
        }

        // Check if the base needs energy
        if let Some(state) = find_base_structure_needing_energy(&room) {
            return state;
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
        if let Some(upgrade_controller_state) = upgrade_controller(&room) {
            return upgrade_controller_state;
        }

        // return idle state if no other states are compatible
        Box::new(IdleState {})
    }

    // Create a generalist with both Carry and Work with one Move per Carry and Work
    fn get_best_worker_body(&self, room: &Room) -> Vec<Part> {
        let mut base_body = vec![];
        let blueprint = vec![
            Part::Move,
            Part::Carry,
            Part::Work,
            Part::Move,
        ];
        let blueprint_cost = blueprint.iter().map(|p: &Part| p.cost()).sum::<u32>();
        let energy_available: u32 = get_total_upgrade_energy(room);
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
