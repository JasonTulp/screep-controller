use crate::screep_states::StateName;
use crate::state_controllers::Specialisation;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::JsValue;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CreepMemory {
    // What state is the creep currently in?
    current_state: StateName,
    // What specialisation is this creep?
    specialisation: Specialisation,
    // Optional field for additional data - skipped during serde operations
    additional_data: Option<String>,
}

impl CreepMemory {
    pub fn new(specialisation: Specialisation) -> Self {
        CreepMemory {
            current_state: StateName::Idle,
            specialisation,
            additional_data: None,
        }
    }

    pub fn current_state(&self) -> &StateName {
        &self.current_state
    }

    pub fn specialisation(&self) -> &Specialisation {
        &self.specialisation
    }

    pub fn additional_data(&self) -> Option<String> {
        self.additional_data.clone()
    }

    pub fn set_current_state(&mut self, state: StateName) {
        self.current_state = state;
    }

    pub fn set_additional_data(&mut self, data: String) {
        self.additional_data = Some(data);
    }
}

impl From<JsValue> for CreepMemory {
    fn from(js_value: JsValue) -> Self {
        from_value(js_value).unwrap_or_else(|_| CreepMemory::new(Specialisation::Unknown))
    }
}

impl From<CreepMemory> for JsValue {
    fn from(memory: CreepMemory) -> Self {
        to_value(&memory).expect("Failed to convert CreepMemory to JsValue")
    }
}
