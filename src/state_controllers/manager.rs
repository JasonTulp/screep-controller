use std::collections::HashMap;
use log::warn;
use screeps::{find, game, objects::Creep, Room, SharedCreepProperties, SpawnOptions};
use crate::info;
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

                // Determine specialisation, and get body parts and memory
                let specialisation = self.get_next_specialty(&spawn.room().unwrap());
                let memory = CreepMemory::new(specialisation.clone());
                let body = match specialisation {
                    Specialisation::Generalist => {
                        let sc = SCGeneralist::new();
                        sc.get_best_worker_body(&spawn.room().unwrap())
                    },
                    // Add more specializations here as needed
                    _ => {
                        let sc = SCGeneralist::new();
                        sc.get_best_worker_body(&spawn.room().unwrap())
                    }
                };
                
                // If we can spawn, spawn a new creep
                if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
                    // create a unique name, spawn.
                    let name_base = game::time();
                    let name = format!("{}-{}", name_base, additional);
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
        info!("Spawning new state controller for creep {}", creep.name());
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

    /// Get the next specialty for a creep based on the current room state
    fn get_next_specialty(&mut self, room: &Room) -> Specialisation {
        // Get all existing specializations in room
        let mut total = 0;
        let mut generalist_count = 0;
        let mut miner_count = 0;
        let mut hauler_count = 0;

        room.find(find::CREEPS, None).iter().for_each(|creep| {
            total += 1;
            match CreepMemory::from(creep.memory()).specialisation() {
                Specialisation::Generalist => generalist_count += 1,
                Specialisation::Miner => miner_count += 1,
                Specialisation::Hauler => hauler_count += 1,
                _ => {}
            }
        });
        info!("Current counts - Generalist: {}, Miner: {}, Hauler: {}",
            generalist_count, miner_count, hauler_count);

        // If there are less than 3 creeps, we need a generalist to spawn
        if total < 2 {
            return Specialisation::Generalist;
        }
        // if generalist_count >= 2 {
        //     // If we have enough generalists, we can spawn a miner or hauler
        //     if miner_count < 1 {
        //         return Specialisation::Miner;
        //     } else if hauler_count < 1 {
        //         return Specialisation::Hauler;
        //     }
        // }
        Specialisation::Generalist
    }
} 