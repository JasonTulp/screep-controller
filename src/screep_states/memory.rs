use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;
use serde::{Deserialize, Serialize};
use crate::screep_states::StateName;
use crate::state_controllers::Specialisation;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreepMemory {
    // What state is the creep currently in?
    current_state: StateName,
    // What specialisation is this creep?
    specialisation: Specialisation,
}

impl CreepMemory {
    pub fn new(specialisation: Specialisation) -> Self {
        CreepMemory {
            current_state: StateName::Idle,
            specialisation,
        }
    }

    pub fn current_state(&self) -> &StateName {
        &self.current_state
    }

    pub fn specialisation(&self) -> &Specialisation {
        &self.specialisation
    }
}

impl From<JsValue> for CreepMemory {
    fn from(js_value: JsValue) -> Self {
        from_value(js_value).unwrap_or_else(|_| {
            CreepMemory::new(Specialisation::Unknown)    
        })
    }
}

impl From<CreepMemory> for JsValue {
    fn from(memory: CreepMemory) -> Self {
        to_value(&memory).expect("Failed to convert CreepMemory to JsValue")
    }
}