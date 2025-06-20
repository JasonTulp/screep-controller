use super::{Specialisation, StateController};
use crate::info;
use crate::screep_states::CreepMemory;
use log::warn;
use screeps::{
    find, game, objects::Creep, Room, SharedCreepProperties, SpawnOptions, StructureObject,
};
use std::collections::HashMap;

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
            let maybe_controller = self.state_controllers.get_mut(&name);
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
        if creep_count < 10 {
            for spawn in game::spawns().values() {
                info!("\n\n\n");
                info!("====> running spawn {}", spawn.name());
                info!(
                    "Energy available: {}",
                    spawn.room().unwrap().energy_available()
                );

                // Determine specialisation, and get body parts and memory
                let specialisation = self.get_next_specialty(&spawn.room().unwrap());
                info!("Next specialisation: {:?}", specialisation);
                let memory = CreepMemory::new(specialisation.clone());
                let controller: Box<dyn StateController> = specialisation.clone().into();
                let body = controller.get_best_worker_body(&spawn.room().unwrap());

                // If we can spawn, spawn a new creep
                if spawn.room().unwrap().energy_available() >= body.iter().map(|p| p.cost()).sum() {
                    // create a unique name, spawn.
                    let name_base = game::time();
                    let name = format!("{:?}-{}-{}", specialisation, name_base, additional);
                    let options = SpawnOptions::new().memory(memory.into());
                    match spawn.spawn_creep_with_options(&body, &name, &options) {
                        Ok(()) => {
                            info!("====> spawn successful\n\n\n");
                            additional += 1
                        },
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
        // Add the new controller to the map
        // Can use into here due to impl on Specialisation
        self.state_controllers.insert(
            creep.name().to_string(),
            memory.specialisation().clone().into(),
        );
    }

    /// Get the next specialty for a creep based on the current room state
    fn get_next_specialty(&mut self, room: &Room) -> Specialisation {
        // check if there is a storage in the room, the hauler and miner combo only work
        // if there is a storage and some containers, but the storage comes last
        // Actually maybe not, the hauler can just fallback to upgrading the controller
        // let storage_exists = room.find(find::STRUCTURES, None)
        //     .iter()
        //     .any(|s| matches!(s, StructureObject::StructureStorage(_)));
        // if (!storage_exists) { return Specialisation::Generalist; }

        // Get all existing specializations in room
        let mut total = 0;
        let mut generalist_count = 0;
        let mut miner_count = 0;
        let mut hauler_count = 0;
        let mut builder_count = 0;
        let mut upgrader_count = 0;

        room.find(find::CREEPS, None).iter().for_each(|creep| {
            total += 1;
            match CreepMemory::from(creep.memory()).specialisation() {
                Specialisation::Generalist => generalist_count += 1,
                Specialisation::Miner => miner_count += 1,
                Specialisation::Hauler => hauler_count += 1,
                Specialisation::Builder => builder_count += 1,
                Specialisation::Upgrader => upgrader_count += 1,
                _ => {}
            }
        });
        // If there are less than 3 creeps, we need a generalist to spawn
        if total < 2 {
            return Specialisation::Generalist;
        }

        let energy_count = room.find(find::SOURCES_ACTIVE, None).len();
        let container_count = room
            .find(find::STRUCTURES, None)
            .iter()
            .filter(|s| matches!(s, StructureObject::StructureContainer(_)))
            .count();
        // set to max energy or container count
        let max_miner_count = energy_count.max(container_count);
        if max_miner_count == 0 {
            // If there are no sources or containers, we can't spawn miners or haulers yet
            return Specialisation::Generalist;
        }

        // Trigger the specialised roles once we have reached a stage where the room can support them
        if generalist_count >= 1 || miner_count + hauler_count >= 2 {
            // If we have enough generalists, we can spawn a miner or hauler
            // Spawn one miner per energy in the room, and alternate miners to haulers
            if miner_count < max_miner_count && miner_count <= hauler_count {
                return Specialisation::Miner;
            } else if hauler_count < max_miner_count {
                return Specialisation::Hauler;
            } else if builder_count <= upgrader_count {
                // If we have enough miners and haulers, we can spawn a builder
                return Specialisation::Builder;
            } else {
                // If we have enough builders, we can spawn an upgrader
                return Specialisation::Upgrader;
            }
        }
        Specialisation::Generalist
    }
}
