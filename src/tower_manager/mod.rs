use log::warn;
use screeps::{find, game, HasHits, HasId, HasPosition, ResourceType, Room, StructureObject, StructureTower};

pub struct TowerManager;

impl TowerManager {
    pub fn new() -> Self {
        TowerManager
    }

    pub fn run_all_towers(&self) {
        for room in game::rooms().values() {
            // Iterate through all towers in the game and run their logic
            for structure in room.find(find::STRUCTURES, None).iter() {
                if let StructureObject::StructureTower(tower) = structure {
                    self.run(&room, &tower);
                }
            }
        }
    }

    /// Run the tower manager logic for the given tower
    fn run(&self, room: &Room, tower: &StructureTower) {
        if tower.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
            return;
        }
        if let Some(target) = self.find_repair_target(room, tower) {
            // Extract the individual structure type that implements Repairable
            match &target {
                StructureObject::StructureRoad(road) => {
                    let _ = tower.repair(road).map_err(|err| {
                        warn!("Tower {} failed to repair road: {}", tower.id(), err);
                    });
                }
                StructureObject::StructureContainer(container) => {
                    let _ = tower.repair(container).map_err(|err| {
                        warn!("Tower {} failed to repair container: {}", tower.id(), err);
                    });
                }
                StructureObject::StructureRampart(rampart) => {
                    let _ = tower.repair(rampart).map_err(|err| {
                        warn!("Tower {} failed to repair rampart: {}", tower.id(), err);
                    });
                }
                StructureObject::StructureWall(wall) => {
                    let _ = tower.repair(wall).map_err(|err| {
                        warn!("Tower {} failed to repair wall: {}", tower.id(), err);
                    });
                }
                StructureObject::StructureExtension(extension) => {
                    let _ = tower.repair(extension).map_err(|err| {
                        warn!("Tower {} failed to repair extension: {}", tower.id(), err);
                    });
                }
                StructureObject::StructureSpawn(spawn) => {
                    let _ = tower.repair(spawn).map_err(|err| {
                        warn!("Tower {} failed to repair spawn: {}", tower.id(), err);
                    });
                }
                StructureObject::StructureTower(tower_target) => {
                    let _ = tower.repair(tower_target).map_err(|err| {
                        warn!("Tower {} failed to repair tower: {}", tower.id(), err);
                    });
                }
                _ => {}
            }
        }
    }

    /// Find a repair target and sort based on distance to the tower
    fn find_repair_target(&self, room: &Room, tower: &StructureTower) -> Option<StructureObject> {
        // Logic to find a structure that needs repairing
        let structures = room.find(find::STRUCTURES, None);
        structures.iter().filter_map(|s| match s {
            StructureObject::StructureRoad(road) if road.hits() < road.hits_max() => {
                Some(s.clone())
            }
            StructureObject::StructureContainer(container)
                if container.hits() < container.hits_max() =>
            {
                Some(s.clone())
            }
            // StructureObject::StructureRampart(rampart)
            //     if rampart.hits() < rampart.hits_max() =>
            // {
            //     Some(s.clone())
            // }
            StructureObject::StructureWall(wall) if wall.hits() < wall.hits_max() => {
                Some(s.clone())
            }
            // StructureObject::StructureExtension(extension)
            //     if extension.hits() < extension.hits_max() =>
            // {
            //     Some(s.clone())
            // }
            // StructureObject::StructureSpawn(spawn) if spawn.hits() < spawn.hits_max() => {
            //     Some(s.clone())
            // }
            // StructureObject::StructureTower(tower) if tower.hits() < tower.hits_max() => {
            //     Some(s.clone())
            // }
            _ => None,
        })
        .min_by_key(|s| tower.pos().get_range_to(s.pos()))
    }
}
