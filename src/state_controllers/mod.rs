mod generalist;
mod hauler;
mod manager;
mod miner;

use log::warn;
use std::cmp::PartialEq;
// Contains core State Controller logic for managing Screep states
use crate::screep_states::*;
use screeps::{find, objects::Creep, Part, Room};
use serde::{Deserialize, Serialize};

use crate::state_controllers::hauler::SCHauler;
use crate::state_controllers::miner::SCMiner;
pub use generalist::SCGeneralist;
pub use manager::SCManager;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Specialisation {
    Unknown,
    Generalist,
    Miner,
    Hauler,
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

impl From<Specialisation> for Box<dyn StateController> {
    fn from(specialisation: Specialisation) -> Self {
        match specialisation {
            Specialisation::Generalist => Box::new(SCGeneralist::new()),
            Specialisation::Miner => Box::new(SCMiner::new()),
            Specialisation::Hauler => Box::new(SCHauler::new()),
            _ => {
                warn!(
                    "!!!! Unknown or unsupported specialisation: {:?} defaulting to Generalist",
                    specialisation
                );
                Box::new(SCGeneralist::new())
            } // Default to Generalist for unknown or unsupported specialisations
        }
    }
}

pub trait StateController {
    /// Get the name of the controller for logging purposes
    fn get_name(&self) -> &'static str;

    /// Run a tick for the given creep and update its state
    fn run_tick(&mut self, creep: &Creep) {
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
    fn get_best_worker_body(&self, _room: &Room) -> Vec<Part>;

    // Count instances of a certain state in the room
    fn count_state_instances(&self, room: &Room, state: &StateName) -> u8 {
        let mut count = 0;
        room.find(find::CREEPS, None).iter().for_each(|creep| {
            if CreepMemory::from(creep.memory()).current_state() == state {
                count += 1;
            }
        });
        count
    }
}
