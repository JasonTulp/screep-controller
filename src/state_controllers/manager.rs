use std::collections::HashMap;
use log::warn;
use screeps::{game, objects::Creep, SharedCreepProperties, SpawnOptions};
use crate::{get_best_worker_body, info};
use super::{StateController, SCGeneralist, Specialisation};
use crate::screep_states::CreepMemory;

/// The SCManager is responsible for managing the state controllers of all creeps in the room.
pub struct SCManager {
    pub state_controllers: HashMap<String, Box<dyn StateController>>,
}

impl SCManager {
    pub fn new() -> Self {
        SCManager {
            state_controllers: HashMap::new(),
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
        let new_controller = match memory.specialisation() {
            Specialisation::Generalist => Box::new(SCGeneralist::new()),
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
} 