use super::{ScreepState, StateNames, TickResult};
use log::warn;
use screeps::action_error_codes::UpgradeControllerErrorCode;
use screeps::{
    constants::ResourceType,
    local::ObjectId,
    objects::{Creep, StructureController},
};

// UpgradeState is used to upgrade the room controller
pub struct UpgradeState {
    controller: ObjectId<StructureController>,
}

impl UpgradeState {
    pub fn new(controller: ObjectId<StructureController>) -> Self {
        UpgradeState { controller }
    }
}

impl ScreepState for UpgradeState {
    fn on_start(&self, creep: &Creep) {
        let _ = creep.say("⬆️", false);
    }

    fn get_state_name(&self) -> &'static str {
        StateNames::Upgrade.into()
    }

    fn tick(&self, creep: &Creep) -> TickResult {
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
}
