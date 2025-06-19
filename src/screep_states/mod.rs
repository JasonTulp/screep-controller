pub use build::BuildState;
pub use feed_structure::FeedStructureState;
pub use harvest::HarvestState;
pub use idle::IdleState;
use log::info;
use screeps::objects::Creep;
use screeps::SharedCreepProperties;
pub use upgrade::UpgradeState;

mod build;
mod feed_structure;
mod harvest;
mod idle;
mod upgrade;

pub enum StateNames {
    Harvest,
    Upgrade,
    Build,
    FeedStructure,
    Idle,
}

impl From<StateNames> for &'static str {
    fn from(state: StateNames) -> Self {
        match state {
            StateNames::Harvest => "Harvest",
            StateNames::Upgrade => "Upgrade",
            StateNames::Build => "Build",
            StateNames::FeedStructure => "FeedStructure",
            StateNames::Idle => "Idle",
        }
    }
}

// What state is this screep in
pub trait ScreepState {
    /// Called when the state is started, can be used to initialize counters or send messages
    fn on_start(&self, creep: &Creep) {
        let _ = creep.say("🌀", false);
    }

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
