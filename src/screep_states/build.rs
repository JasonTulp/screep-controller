use super::{ScreepState, TickResult};
use crate::state_controllers::StateController;
use log::warn;
use screeps::action_error_codes::BuildErrorCode;
use screeps::{
    constants::ResourceType,
    objects::{ConstructionSite, Creep},
    prelude::*,
    HasPosition,
};

pub struct BuildState {
    construction_site: ConstructionSite,
}

impl BuildState {
    pub fn new(construction_site: ConstructionSite) -> Self {
        BuildState { construction_site }
    }
}

impl ScreepState for BuildState {
    fn on_start(&self, creep: &Creep, state_controller: &mut StateController) {
        state_controller.build_creeps += 1;
        let _ = creep.say("⚒️", false);
    }

    fn get_state_name(&self) -> &'static str {
        "Build"
    }

    fn tick(&mut self, creep: &Creep) -> TickResult {
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return TickResult::Exit;
        }
        // Move to construction site. THis is to free up the resource source if others wanna get in
        if !creep.pos().is_near_to(self.construction_site.pos()) {
            let _ = creep.move_to(self.construction_site.pos());
            return TickResult::Continue;
        }

        match creep.build(&self.construction_site) {
            Ok(_) => {
                // Successfully built
                TickResult::Continue
            }
            Err(e) => {
                // Handle the error based on the error code
                return match e {
                    BuildErrorCode::NotInRange => {
                        let _ = creep.move_to(self.construction_site.clone());
                        warn!("creep {} is not in range to build", creep.name());
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
        // Decrease the count of build creeps when exiting this state
        state_controller.build_creeps = state_controller.build_creeps.saturating_sub(1);
    }
}
