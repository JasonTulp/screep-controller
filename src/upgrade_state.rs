use crate::{warn};
use crate::state_machine::{ScreepState, StateController, TickResult};
use screeps::{
    constants::{ResourceType},
    local::ObjectId,
    objects::{Creep, StructureController},
    HasPosition,
    prelude::*,
};
use screeps::action_error_codes::{UpgradeControllerErrorCode};

pub struct UpgradeState {
    controller: ObjectId<StructureController>,
}

impl UpgradeState {
    pub fn new(controller: ObjectId<StructureController>) -> Self {
        UpgradeState { controller }
    }
}

impl ScreepState for UpgradeState {
    fn on_start(&self, creep: &Creep, state_controller: &mut StateController) {
        state_controller.upgrade_creeps += 1;
        let _ = creep.say("⬆️", false);
    }

    fn tick(&mut self, creep: &Creep) -> TickResult {
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return TickResult::Exit;
        }
        let Some(controller) = self.controller.resolve() else {
            return TickResult::Exit;
        };
        match creep.upgrade_controller(&controller) {
            Ok(_) => {
                // Successfully upgraded the controller
                TickResult::Continue
            }
            Err(e) => {
                // Handle the error based on the error code
                return match e {
                    UpgradeControllerErrorCode::NotInRange => {
                        let _ = creep.move_to(&controller);
                        TickResult::Continue
                    }
                    _ => {
                        warn!("couldn't upgrade: {:?}", e);
                        TickResult::Exit
                    }
                };
            }
        }
    }

    fn on_exit(&self, state_controller: &mut StateController) {
        // Decrease the count of upgrade creeps when exiting this state
        state_controller.upgrade_creeps = state_controller.upgrade_creeps.saturating_sub(1);
    }
}
