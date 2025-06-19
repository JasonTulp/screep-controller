use super::{ScreepState, StateName, TickResult};
use screeps::action_error_codes::TransferErrorCode;
use screeps::{constants::ResourceType, local::ObjectId, objects::Creep, prelude::*};
use wasm_bindgen::JsCast;

pub struct FeedStructureState<T: Transferable + MaybeHasId + JsCast> {
    structure: ObjectId<T>,
}

impl<T: Transferable + MaybeHasId + JsCast> FeedStructureState<T> {
    pub fn new(structure: ObjectId<T>) -> Self {
        FeedStructureState { structure }
    }
}

impl<T: Transferable + MaybeHasId + JsCast> ScreepState for FeedStructureState<T> {
    fn on_start(&self, creep: &Creep) {
        let _ = creep.say("💪", false);
        self.update_memory(creep);
    }

    fn get_state_name(&self) -> &'static str {
        StateName::FeedStructure.into()
    }

    fn tick(&self, creep: &Creep) -> TickResult {
        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return TickResult::Exit;
        }
        let Some(structure) = self.structure.resolve() else {
            return TickResult::Exit;
        };
        match creep.transfer(&structure, ResourceType::Energy, None) {
            Ok(_) => {
                // Successfully transferred to the structure
                TickResult::Continue
            }
            Err(e) => {
                // Handle the error based on the error code
                return match e {
                    TransferErrorCode::NotInRange => {
                        let _ = creep.move_to(&structure);
                        TickResult::Continue
                    }
                    _ => TickResult::Exit,
                };
            }
        }
    }
}
