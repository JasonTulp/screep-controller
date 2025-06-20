use super::{Specialisation, StateController};
use crate::screep_states::*;
use crate::utils::prelude::*;
use crate::utils;
use screeps::{
    objects::Creep, prelude::*, Part, Room,
};
use crate::utils::upgrade_controller;

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
        // Attempt to find energy if we need it
        if let Some(energy_state) = find_energy(&room, creep, EnergyAuthority::StorageOrContainers) {
            return energy_state;
        }
        // upgrade controller is my main objective!
        if let Some(upgrade_controller_state) = upgrade_controller(&room) {
            return upgrade_controller_state;
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
