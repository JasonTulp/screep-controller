pub use build::BuildState;
pub use feed_structure::FeedStructureState;
pub use harvest::HarvestState;
pub use idle::IdleState;
use log::info;
use screeps::objects::Creep;
use screeps::SharedCreepProperties;
use serde::{Deserialize, Serialize};
pub use upgrade::UpgradeState;

mod build;
mod feed_structure;
mod harvest;
mod idle;
mod memory;
mod upgrade;
pub use memory::CreepMemory;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum StateName {
    Harvest,
    Upgrade,
    Build,
    FeedStructure,
    Idle,
}

impl From<StateName> for &'static str {
    fn from(state: StateName) -> Self {
        match state {
            StateName::Harvest => "Harvest",
            StateName::Upgrade => "Upgrade",
            StateName::Build => "Build",
            StateName::FeedStructure => "FeedStructure",
            StateName::Idle => "Idle",
        }
    }
}

impl From<&'static str> for StateName {
    fn from(state: &'static str) -> Self {
        match state {
            "Harvest" => StateName::Harvest,
            "Upgrade" => StateName::Upgrade,
            "Build" => StateName::Build,
            "FeedStructure" => StateName::FeedStructure,
            _ => StateName::Idle,
        }
    }
}

// What state is this screep in
pub trait ScreepState {
    fn update_memory(&self, creep: &Creep) {
        let memory: CreepMemory = creep.memory().into();
        let updated = CreepMemory {
            current_state: self.get_state_name().into(),
            specialisation: memory.specialisation,
        };
        creep.set_memory(&updated.into());
    }

    /// Called when the state is started, can be used to initialize counters or send messages
    fn on_start(&self, creep: &Creep);

    /// Log the current state of the creep for debugging purposes
    fn log_state(&self, creep: &Creep) {
        info!(
            "-> Creep {} is in {} state.",
            creep.name(),
            self.get_state_name()
        );
    }

    /// Get the name of the state for logging purposes
    fn get_state_name(&self) -> &'static str;

    /// Run a tick for the given creep and return the result
    fn tick(&self, creep: &Creep) -> TickResult;

    /// Called when the state is exited, can be used to clean up or reset counters
    fn on_exit(&self) {
        return;
    }
}

// Result from a tick
pub enum TickResult {
    // Keep the state as-is
    Continue,
    // Change to a specific state
    #[allow(dead_code)]
    ChangeState(Box<dyn ScreepState>),
    // exit and choose a state based on current needs
    Exit,
}
