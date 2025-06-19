use super::{ScreepState, StateName, TickResult};
use screeps::objects::Creep;

/// Idle state can be used as a fallback when no other state is applicable
/// This state will do nothing and constantly search for a better state on each tick
pub struct IdleState;

impl ScreepState for IdleState {
    fn on_start(&self, creep: &Creep) {
        let _ = creep.say("💤", false);
        self.update_memory(creep);
    }

    fn get_state_name(&self) -> &'static str {
        StateName::Idle.into()
    }

    fn tick(&self, _creep: &Creep) -> TickResult {
        // Do nothing, just idle until new state can be chosen
        TickResult::Exit
    }
}
