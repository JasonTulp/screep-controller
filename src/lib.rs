extern crate core;

use js_sys::{JsString, Object, Reflect};
use log::*;
use screeps::game;
use std::{cell::RefCell, collections::HashSet};
use wasm_bindgen::prelude::*;

mod logging;
mod screep_states;
mod state_controllers;
mod tower_manager;
mod utils;

use crate::state_controllers::SCManager;
use tower_manager::TowerManager;

// this is one way to persist data between ticks within Rust's memory, as opposed to
// keeping state in memory on game objects - but will be lost on global resets!
thread_local! {
    static STATE_MANAGER: RefCell<SCManager> = RefCell::new(SCManager::new());
}

static INIT_LOGGING: std::sync::Once = std::sync::Once::new();

// add wasm_bindgen to any function you would like to expose for call from js
// to use a reserved name as a function name, use `js_name`:
#[wasm_bindgen(js_name = loop)]
pub fn game_loop() {
    INIT_LOGGING.call_once(|| {
        // show all output of Info level, adjust as needed
        // logging::setup_logging(logging::Info);
        // logging::setup_logging(logging::Warn);
        logging::setup_logging(logging::Trace);
    });

    // debug!("loop starting! CPU: {}", game::cpu::get_used());
    STATE_MANAGER.with(|state_manager_refcell| {
        let mut state_manager = state_manager_refcell.borrow_mut();
        // run the tick for all state controllers
        state_manager.run();
    });

    // Run all towers to repair some shit
    TowerManager::new().run_all_towers();

    // memory cleanup; memory gets created for all creeps upon spawning, and any time move_to
    // is used; this should be removed if you're using RawMemory/serde for persistence
    if game::time() % 1000 == 0 {
        info!("running memory cleanup");
        let mut alive_creeps = HashSet::new();
        // add all living creep names to a hashset
        for creep_name in game::creeps().keys() {
            alive_creeps.insert(creep_name);
        }

        // grab `Memory.creeps` (if it exists)
        #[allow(deprecated)]
        if let Ok(memory_creeps) = Reflect::get(&screeps::memory::ROOT, &JsString::from("creeps")) {
            // convert from JsValue to Object
            let memory_creeps: Object = memory_creeps.unchecked_into();
            // iterate memory creeps
            for creep_name_js in Object::keys(&memory_creeps).iter() {
                // convert to String (after converting to JsString)
                let creep_name = String::from(creep_name_js.dyn_ref::<JsString>().unwrap());

                // check the HashSet for the creep name, deleting if not alive
                if !alive_creeps.contains(&creep_name) {
                    info!("deleting memory for dead creep {}", creep_name);
                    let _ = Reflect::delete_property(&memory_creeps, &creep_name_js);
                }
            }
        }
    }

    // info!("done! cpu: {}", game::cpu::get_used())
}
