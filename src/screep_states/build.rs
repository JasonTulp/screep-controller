use super::{ScreepState, StateName, TickResult};
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
    fn on_start(&self, creep: &Creep) {
        let _ = creep.say("⚒️", false);
        self.update_memory(creep);
    }

    fn get_state_name(&self) -> StateName {
        StateName::Build
    }

    fn tick(&self, creep: &Creep) -> TickResult {
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
}
