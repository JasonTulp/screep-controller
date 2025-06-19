use crate::screep_states::CreepMemory;

use super::{ScreepState, StateName, TickResult};
use log::{info, warn};
use screeps::{
    constants::ResourceType,
    local::ObjectId,
    objects::{Creep, Source},
    prelude::*,
};
use wasm_bindgen::JsCast;

/// Harvest energy from the source
pub struct WithdrawState<T: Withdrawable + MaybeHasId + JsCast> {
    structure: ObjectId<T>,
}

impl<T: Withdrawable + MaybeHasId + JsCast> WithdrawState<T> {
    pub fn new(structure: ObjectId<T>) -> Self {
        WithdrawState { structure }
    }
}

impl<T: Withdrawable + MaybeHasId + JsCast> ScreepState for WithdrawState<T> {
    fn on_start(&self, creep: &Creep) {
        let _ = creep.say("📤", false);
        self.update_state_memory(creep);
    }

    fn get_state_name(&self) -> StateName{
        StateName::Withdraw
    }

    fn tick(&self, creep: &Creep) -> TickResult {
        // Check if we have any free capacity to harvest energy
        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
            return TickResult::Exit;
        }
        let Some(structure) = self.structure.resolve() else {
            return TickResult::Exit;
        };

        if creep.pos().is_near_to(structure.pos()) {
            if creep.withdraw(&structure, ResourceType::Energy, None).is_err() {
                warn!("couldn't harvest for some unknown reason");
                return TickResult::Exit;
            };
        } else {
            let _ = creep.move_to(&structure);
        }

        TickResult::Continue
    }
}
