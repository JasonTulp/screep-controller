use super::{ScreepState, StateName, TickResult};
use log::warn;
use screeps::{
    constants::ResourceType,
    local::ObjectId,
    objects::{Creep, Source},
    prelude::*,
};

/// Harvest energy from the source
pub struct HarvestState {
    source: ObjectId<Source>,
}

impl HarvestState {
    pub fn new(source: ObjectId<Source>) -> Self {
        HarvestState { source }
    }
}

impl ScreepState for HarvestState {
    fn on_start(&self, creep: &Creep) {
        let _ = creep.say("⚡", false);
    }

    fn get_state_name(&self) -> &'static str {
        StateName::Harvest.into()
    }

    fn tick(&self, creep: &Creep) -> TickResult {
        // Check if we have any free capacity to harvest energy
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            return TickResult::Exit;
        }
        let Some(source) = self.source.resolve() else {
            return TickResult::Exit;
        };

        if creep.pos().is_near_to(source.pos()) {
            if creep.harvest(&source).is_err() {
                warn!("couldn't harvest for some unknown reason");
                return TickResult::Exit;
            };
        } else {
            let _ = creep.move_to(&source);
        }

        TickResult::Continue
    }
}
