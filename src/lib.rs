extern crate core;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
};

use crate::state_machine::StateController;
use js_sys::{JsString, Object, Reflect};
use log::*;
use screeps::{
    constants::{Part, ResourceType},
    enums::StructureObject,
    find, game,
    objects::Room,
    prelude::*,
};
use state_machine::ScreepState;
use wasm_bindgen::prelude::*;

mod build_state;
mod feed_structure_state;
mod harvest_state;
mod logging;
mod state_machine;
mod upgrade_state;

// this is one way to persist data between ticks within Rust's memory, as opposed to
// keeping state in memory on game objects - but will be lost on global resets!
thread_local! {
    static CREEP_STATES: RefCell<HashMap<String, Box<dyn ScreepState>>> = RefCell::new(HashMap::new());
    static STATE_CONTROLLER: RefCell<StateController> = RefCell::new(StateController::new());
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

    debug!("loop starting! CPU: {}", game::cpu::get_used());

    STATE_CONTROLLER.with(|state_controller_refcell| {
        let mut sc = state_controller_refcell.borrow_mut();
        // mutably borrow the creep_targets refcell, which is holding our creep target locks
        // in the wasm heap
        CREEP_STATES.with(|creep_targets_refcell| {
            let mut creep_targets = creep_targets_refcell.borrow_mut();
            // debug!("running creeps");
            for creep in game::creeps().values() {
                sc.run_tick(&creep, &mut creep_targets);
            }
        });
    });

    debug!("running spawns");
    let mut additional = 0;
    let creep_count = game::creeps().values().count();
    // info!("creep count: {}", creep_count);
    if creep_count < 5 {
        for spawn in game::spawns().values() {
            info!("running spawn {}", spawn.name());
            info!(
                "Energy available: {}",
                spawn.room().unwrap().energy_available()
            );

            let body = get_best_worker_body(&spawn.room().unwrap());
            if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
                // create a unique name, spawn.
                let name_base = game::time();
                let name = format!("{}-{}", name_base, additional);
                match spawn.spawn_creep(&body, &name) {
                    Ok(()) => additional += 1,
                    Err(e) => warn!("couldn't spawn: {:?}", e),
                }
            }
        }
    }

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

    info!("done! cpu: {}", game::cpu::get_used())
}

/// This function returns the best worker body based on the current game state.
fn get_best_worker_body(room: &Room) -> Vec<Part> {
    let mut base_body = vec![Part::Move, Part::Move, Part::Carry, Part::Work];
    let energy_available: u32 = get_total_upgrade_energy(room);
    info!("Total available: {}", energy_available);
    let mut cost = base_body.iter().map(|p| p.cost()).sum::<u32>();
    while cost < energy_available {
        if cost + Part::Work.cost() <= energy_available {
            base_body.push(Part::Work);
            cost += Part::Work.cost();
        }

        if cost + Part::Move.cost() <= energy_available {
            base_body.push(Part::Move);
            cost += Part::Move.cost();
        }

        if cost + Part::Carry.cost() <= energy_available {
            base_body.push(Part::Carry);
            cost += Part::Carry.cost();
        }
    }

    return base_body;
}

/// The max capacity of energy available for upgrades in a room.
/// This is the sum of the spawns and any extensions in the room.
fn get_total_upgrade_energy(room: &Room) -> u32 {
    let mut energy_available: u32 = 0;
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureSpawn(spawn) = structure {
            energy_available += spawn.store().get_capacity(Some(ResourceType::Energy));
        } else if let StructureObject::StructureExtension(extension) = structure {
            energy_available += extension.store().get_capacity(Some(ResourceType::Energy));
        }
    }
    energy_available
}
