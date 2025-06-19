mod generalist;
mod manager;

// Contains core State Controller logic for managing Screep states
use crate::screep_states::*;
use screeps::{objects::Creep, prelude::*, Part, Room, SpawnOptions};
use serde::{Deserialize, Serialize};

pub use generalist::SCGeneralist;
pub use manager::SCManager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Specialisation {
    Unknown,
    Generalist,
    Miner,
    Hauler
}

impl From<Specialisation> for &'static str {
    fn from(state: Specialisation) -> Self {
        match state {
            Specialisation::Unknown => "Unknown",
            Specialisation::Generalist => "Generalist",
            Specialisation::Miner => "Miner",
            Specialisation::Hauler => "Hauler",
        }
    }
}

pub trait StateController {
    /// Get the name of the controller for logging purposes
    fn get_name(&self) -> &'static str;

    /// Run a tick for the given creep and update its state
    fn run_tick(&mut self, creep: &Creep) {
        let name = creep.name();
        match self.current_state().tick(creep) {
            TickResult::Continue => {
                // Continue running the current state
                return;
            }
            TickResult::ChangeState(new_state) => {
                // Exit the current state
                self.current_state().on_exit();
                new_state.on_start(creep);
                new_state.log_state(creep);
                // set creep state to the new state
                self.set_current_state(new_state);
            }
            TickResult::Exit => {
                // Exit the current state and remove it from the map
                self.current_state().on_exit();
                let new_state: Box<dyn ScreepState> = self.choose_next_state(creep);
                new_state.on_start(creep);
                new_state.log_state(creep);
                self.set_current_state(new_state);
            }
        }
    }

    // What is the current state of the controller
    fn current_state(&self) -> &Box<dyn ScreepState>;

    /// Set the current state of the controller
    fn set_current_state(&mut self, state: Box<dyn ScreepState>);

    /// Choose the next state based on the current needs of the room
    fn choose_next_state(&mut self, creep: &Creep) -> Box<dyn ScreepState>;

    /// Get the best worker body for the current state controller
    fn get_best_worker_body(&self, _room: &Room, _max_energy: u32) -> Vec<Part> {
        vec![Part::Move, Part::Move, Part::Carry, Part::Work]
    }

    fn get_memory(&self) -> CreepMemory;
}
