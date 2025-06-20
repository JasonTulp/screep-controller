use super::{Specialisation, StateController};
use crate::screep_states::*;
use crate::utils::prelude::*;
use screeps::{
    objects::Creep, prelude::*, Part,
};
use crate::utils::upgrade_controller;

/// Builder State Controller for bmanaging a builder creep
pub struct SCBuilder {
    pub current_state: Box<dyn ScreepState>,
}

impl SCBuilder {
    pub fn new() -> Self {
        SCBuilder {
            current_state: Box::new(IdleState {}),
        }
    }
}

impl StateController for SCBuilder {
    fn get_name(&self) -> &'static str {
        Specialisation::Generalist.into()
    }

    fn get_blueprint(&self) -> Vec<Part> {
        vec![
            Part::Move,
            Part::Carry,
            Part::Work,
            Part::Work,
            Part::Move,
            Part::Move,
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

        // Attempt to find energy if we need it
        if let Some(energy_state) = find_energy(&room, creep, EnergyAuthority::StorageOrContainers) {
            return energy_state;
        }

        // Find something to build
        if let Some(site) = find_nearest_construction_site(creep, &room) {
            return Box::new(BuildState::new(site.clone()));
        }

        // upgrade controller if nothing to build
        if let Some(upgrade_controller_state) = upgrade_controller(&room) {
            return upgrade_controller_state;
        }

        // return idle state if no other states are compatible
        Box::new(IdleState {})
    }
}
