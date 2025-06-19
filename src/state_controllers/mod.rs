// Contains core State Controller logic for managing Screep states
use crate::screep_states::StateName;
use crate::screep_states::*;
use crate::utils::prelude::*;
use crate::{get_best_worker_body, info};
use log::warn;
use screeps::{constants::ResourceType, enums::StructureObject, find, game, objects::Creep, prelude::*, Part, Room, SpawnOptions};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Specialisation {
    Unknown,
    Generalist,
    Miner,
    Hauler
}

impl From<Specialisation> for &'static str {
    fn from(state: Specialisation) -> Self {
        match state {
            Specialisation::Unknown => "Unknown",
            Specialisation::Generalist => "Generalist",
            Specialisation::Miner => "Miner",
            Specialisation::Hauler => "Hauler",
        }
    }
}

/// The SCManager is responsible for managing the state controllers of all creeps in the room.
pub struct SCManager {
    pub state_controllers: HashMap<String, Box<dyn StateController>>,
    pub specialty_count: HashMap<String, u8>,
}

impl SCManager {
    pub fn new() -> Self {
        SCManager {
            state_controllers: HashMap::new(),
            specialty_count: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        self.run_spawns();
        // Run the tick for all state controllers
        self.run_tick_for_all();
    }

    pub fn run_tick_for_all(&mut self) {
        for creep in game::creeps().values() {
            let name = creep.name();
            let mut maybe_controller = self.state_controllers.get_mut(&name);
            if let Some(controller) = maybe_controller {
                controller.run_tick(&creep);
            } else {
                self.spawn_new_controller(&creep);
            }
        }
    }

    /// Check if we need to spawn any more creeps, and trigger spawn if we can
    pub fn run_spawns(&mut self) {
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

                // TODO determine which specialist it is, then what body based on specialist
                let body = get_best_worker_body(&spawn.room().unwrap());
                if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
                    // create a unique name, spawn.
                    let name_base = game::time();
                    let name = format!("{}-{}", name_base, additional);
                    let controller = SCGeneralist::new();
                    let memory = controller.get_memory();
                    let options = SpawnOptions::new()
                        .memory(memory.into());
                    match spawn.spawn_creep_with_options(&body, &name, &options) {
                        Ok(()) => additional += 1,
                        Err(e) => warn!("couldn't spawn: {:?}", e),
                    }
                }
            }
        }
    }

    /// Spawn a new state controller for the given creep
    /// This is where we control how many of each controller we need
    fn spawn_new_controller(&mut self, creep: &Creep) {
        warn!("Spawning new state controller for creep {}", creep.name());
        let memory = creep.memory();
        let memory: CreepMemory = memory.into();
        info!("Creep memory: {:?}", memory);
        let specialisation = "Generalist"; //creep.memory().is_object().unwrap_or("Generalist");
        let new_controller = match specialisation {
            "Generalist" => Box::new(SCGeneralist::new()),
            // Add more specializations here as needed
            _ => Box::new(SCGeneralist::new()),
        };
        // Add the new controller to the map
        self.state_controllers
            .insert(creep.name().to_string(), new_controller);
    }

    fn count_specialties(&mut self) {
        // TODO Count specialties and key based on room
    }

    pub fn get_specialty_count(&self, creep_name: &str) -> u8 {
        self.specialty_count.get(creep_name).unwrap_or(&0).clone()
    }
}

pub trait StateController {
    /// Get the name of the controller for logging purposes
    fn get_name(&self) -> &'static str;

    /// Run a tick for the given creep and update its state
    fn run_tick(&mut self, creep: &Creep) {
        let name = creep.name();
        match self.current_state().tick(creep) {
            TickResult::Continue => {
                // Continue running the current state
                return;
            }
            TickResult::ChangeState(new_state) => {
                // Exit the current state
                self.current_state().on_exit();
                new_state.on_start(creep);
                new_state.log_state(creep);
                // set creep state to the new state
                self.set_current_state(new_state);
            }
            TickResult::Exit => {
                // Exit the current state and remove it from the map
                self.current_state().on_exit();
                let new_state: Box<dyn ScreepState> = self.choose_next_state(creep);
                new_state.on_start(creep);
                new_state.log_state(creep);
                self.set_current_state(new_state);
            }
        }
    }

    // What is the current state of the controller
    fn current_state(&self) -> &Box<dyn ScreepState>;

    /// Set the current state of the controller
    fn set_current_state(&mut self, state: Box<dyn ScreepState>);

    /// Choose the next state based on the current needs of the room
    fn choose_next_state(&mut self, creep: &Creep) -> Box<dyn ScreepState>;

    /// Get the best worker body for the current state controller
    fn get_best_worker_body(&self, _room: &Room, _max_energy: u32) -> Vec<Part> {
        vec![Part::Move, Part::Move, Part::Carry, Part::Work]
    }

    fn get_memory(&self) -> CreepMemory;
}

/// Generalist State Controller for managing a sawdcreep that performs a variety of tasks
pub struct SCGeneralist {
    pub current_state: Box<dyn ScreepState>,
    pub creep_memory: CreepMemory,
}

impl SCGeneralist {
    pub fn new() -> Self {
        SCGeneralist {
            current_state: Box::new(IdleState {}),
            creep_memory: CreepMemory::new(Specialisation::Generalist),
        }
    }
}

impl StateController for SCGeneralist {
    fn get_name(&self) -> &'static str {
        "Generalist"
    }

    fn current_state(&self) -> &Box<dyn ScreepState> {
        &self.current_state
    }

    fn set_current_state(&mut self, state: Box<dyn ScreepState>) {
        self.current_state = state;
    }

    fn choose_next_state(&mut self, creep: &Creep) -> Box<dyn ScreepState> {
        let room = creep.room().expect("couldn't resolve creep room");
        let energy = creep.store().get_used_capacity(Some(ResourceType::Energy));
        if energy == 0 {
            // Attempt to find some sources to harvest
            if let Some(source) = find_nearest_object(creep, &room, find::SOURCES_ACTIVE) {
                info!("Starting Harvest State for creep {}", creep.name());
                return Box::new(HarvestState::new(source));
            } else {
                warn!("No sources found for creep {}", creep.name());
                return Box::new(IdleState {});
            }
        }

        // Check if the base needs energy
        let upgrade_energy = get_total_upgrade_energy(&room);
        let energy_available = room.energy_available();
        if energy_available < upgrade_energy {
            // Find a structure to feed energy to
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureSpawn(spawn) = structure {
                    if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        info!(
                            "Starting Feed spawn Structure State for creep {}",
                            creep.name()
                        );
                        return Box::new(
                            FeedStructureState::<screeps::objects::StructureSpawn>::new(spawn.id()),
                        );
                    }
                } else if let StructureObject::StructureExtension(extension) = structure {
                    if extension
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        > 0
                    {
                        info!(
                            "Starting Feed Extension Structure State for creep {}",
                            creep.name()
                        );
                        return Box::new(
                            FeedStructureState::<screeps::objects::StructureExtension>::new(
                                extension.id(),
                            ),
                        );
                    }
                }
            }
        }

        // limit build creeps to 2, only build if we have an upgrade creep
        // TODO
        // if sc_manager.get_specialty_count(StateName::Build.into()) < 2 &&
        //     sc_manager.get_specialty_count(StateName::Upgrade.into()) > 0 {
        //     if let Some(site) = find_nearest_construction_site(creep, &room) {
        //         return Box::new(BuildState::new(site.clone()));
        //     }
        // }

        // Check if we have energy, if we do, upgrade controller
        if energy > 0 {
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureController(controller) = structure {
                    info!("Starting Upgrade State for creep {}", creep.name());
                    return Box::new(UpgradeState::new(controller.id()));
                }
            }
        }

        // return idle state if no other states are compatible
        info!("Starting Idle State for creep {}", creep.name());
        Box::new(IdleState {})
    }

    fn get_memory(&self) -> CreepMemory {
        self.creep_memory.clone()
    }
}
