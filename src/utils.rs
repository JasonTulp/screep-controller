use std::cmp::PartialEq;
// Helpful utility functions for the project.
use screeps::{find, ConstructionSite, Creep, FindConstant, HasId, HasPosition, ObjectId, Position, ResourceType, Room, StructureContainer, StructureObject, StructureStorage, StructureTower};
use crate::screep_states::*;

pub mod prelude {
    pub use super::{
        find_nearest_construction_site,
        get_total_upgrade_energy,
        find_closest_tower_needing_energy, find_container_with_most_energy,
        find_closest_container_to_position, find_base_structure_needing_energy,
        find_object_at_index, EnergyAuthority, find_energy,
    };
}

// Gets the nearest object based on distance from the creep
pub fn find_nearest_object<R>(
    // reference position
    position: &Position,
    room: &Room,
    find_const: impl FindConstant<Item = R>,
) -> Option<ObjectId<R>>
where
    R: HasPosition + Clone + HasId,
{
    let objects = room.find(find_const, None);
    if objects.is_empty() {
        return None;
    }
    // Find the nearest object
    let nearest = objects
        .iter()
        .min_by_key(|obj| position.pos().get_range_to(obj.pos()))?;

    Some(nearest.clone().id())
}

/// Gets an object at a specific index in the room's find results.
pub fn find_object_at_index<R>(
    room: &Room,
    index: u8,
    find_const: impl FindConstant<Item = R>,
) -> Option<ObjectId<R>>
where
    R: HasPosition + Clone + HasId,
{
    let objects = room.find(find_const, None);
    if objects.is_empty() || index as usize >= objects.len() {
        return None;
    }
    // Get the object at the specified index
    let object = &objects[index as usize];
    Some(object.clone().id())
}

/// Get the nearest construction site based on distance from the creep
/// For some reason construction sites are not wrapped in ObjectId, so we can't use the same function as above
pub fn find_nearest_construction_site(creep: &Creep, room: &Room) -> Option<ConstructionSite> {
    let sites = room.find(find::CONSTRUCTION_SITES, None);
    if sites.is_empty() {
        return None;
    }
    // Find the nearest construction site
    let nearest_site = sites
        .iter()
        .min_by_key(|site| creep.pos().get_range_to(site.pos()))?;

    Some(nearest_site.clone())
}

/// The max capacity of energy available for upgrades in a room.
/// This is the sum of the spawns and any extensions in the room.
pub fn get_total_upgrade_energy(room: &Room) -> u32 {
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

#[derive(PartialEq)]
pub enum EnergyAuthority {
    // Can only pull energy from Storage
    StorageOnly,
    // Can only pull energy from Storage and containers, not the source
    StorageOrContainers,
    // Can pull energy from anywhere
    All,
}

/// Called by a creep to find energy, prioritising storage, then containers, then sources
/// If authority is set to 0, it will only look for storage. etc
pub fn find_energy(room: &Room, creep: &Creep, authority: EnergyAuthority) -> Option<Box<dyn ScreepState>> {
    // Check if we even need energy
    let energy = creep.store().get_used_capacity(Some(ResourceType::Energy));
    if energy > 0 {
        return None;
    }

    // Find the closest storage with energy to withdraw from
    if let Some(storage_id) = find_closest_storage(room, creep) {
        return Some(Box::new(WithdrawState::new(storage_id)));
    }
    if authority == EnergyAuthority::StorageOnly {
        return None;
    }

    // Find the closest container with energy to drain
    if let Some(container_id) = find_closest_container_with_energy(&room, creep) {
        return Some(Box::new(WithdrawState::new(container_id)));
    }
    if authority == EnergyAuthority::StorageOrContainers {
        return None;
    }

    // Otherwise, attempt to find some sources to harvest
    if let Some(source) = find_nearest_object(&creep.pos(), &room, find::SOURCES_ACTIVE) {
        return Some(Box::new(HarvestState::new(source)));
    }

    None
}


/// Find the closest storage with energy to withdraw from
pub fn find_closest_storage(room: &Room, creep: &Creep) -> Option<ObjectId<StructureStorage>> {
    let mut closest_storage: Option<ObjectId<StructureStorage>> = None;
    let mut min_distance = u32::MAX;
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureStorage(storage) = structure {
            if storage
                .store()
                .get_used_capacity(Some(ResourceType::Energy))
                > 0
            {
                let distance = creep.pos().get_range_to(storage.pos());
                if distance < min_distance {
                    min_distance = distance;
                    closest_storage = Some(storage.id());
                }
            }
        }
    }
    closest_storage
}

/// Find the closest container with energy to withdraw from
pub fn find_closest_container_with_energy(room: &Room, creep: &Creep) -> Option<ObjectId<StructureContainer>> {
    let mut closest_container: Option<ObjectId<StructureContainer>> = None;
    let mut min_distance = u32::MAX;
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureContainer(container) = structure {
            if container
                .store()
                .get_used_capacity(Some(ResourceType::Energy))
                > 0
            {
                let distance = creep.pos().get_range_to(container.pos());
                if distance < min_distance {
                    min_distance = distance;
                    closest_container = Some(container.id());
                }
            }
        }
    }
    closest_container
}

/// Find the container with the most energy (useful for haulers)
pub fn find_container_with_most_energy(room: &Room) -> Option<ObjectId<StructureContainer>> {
    let mut best_container: Option<ObjectId<StructureContainer>> = None;
    let mut max_energy = 0;
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureContainer(container) = structure {
            if container
                .store()
                .get_used_capacity(Some(ResourceType::Energy))
                > 0
            {
                let energy_in_container = container
                    .store()
                    .get_used_capacity(Some(ResourceType::Energy));
                if energy_in_container > max_energy {
                    max_energy = energy_in_container;
                    best_container = Some(container.id());
                }
            }
        }
    }
    best_container
}

/// Find the closest tower that needs energy
pub fn find_closest_tower_needing_energy(room: &Room, creep: &Creep) -> Option<ObjectId<StructureTower>> {
    let mut closest_tower: Option<ObjectId<StructureTower>> = None;
    let mut min_distance = u32::MAX;
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureTower(tower) = structure {
            if tower.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                let distance = creep.pos().get_range_to(tower.pos());
                if distance < min_distance {
                    min_distance = distance;
                    closest_tower = Some(tower.id());
                }
            }
        }
    }
    closest_tower
}

/// Find the closest container to a specific position (useful for miners finding containers near their source)
pub fn find_closest_container_to_position(room: &Room, position: &Position) -> Option<StructureContainer> {
    let mut closest_container: Option<screeps::objects::StructureContainer> = None;
    let mut closest_distance = u32::MAX;
    
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureContainer(container) = structure {
            let distance = position.get_range_to(container.pos());
            if distance < closest_distance {
                closest_distance = distance;
                closest_container = Some(container.clone());
            }
        }
    }
    closest_container
}

/// Find a structure that needs energy when the base energy is low (spawns and extensions)
pub fn find_base_structure_needing_energy(room: &Room) -> Option<Box<dyn crate::screep_states::ScreepState>> {
    // Check if the base needs energy
    let upgrade_energy = get_total_upgrade_energy(room);
    let energy_available = room.energy_available();
    if energy_available >= upgrade_energy {
        return None;
    }

    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureSpawn(spawn) = structure {
            if spawn.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                return Some(Box::new(FeedStructureState::<screeps::objects::StructureSpawn>::new(spawn.id())));
            }
        } else if let StructureObject::StructureExtension(extension) = structure {
            if extension
                .store()
                .get_free_capacity(Some(ResourceType::Energy))
                > 0
            {
                return Some(Box::new(
                    FeedStructureState::<screeps::objects::StructureExtension>::new(
                        extension.id(),
                    ),
                ));
            }
        }
    }
    None
}

// Find the controller and return an Upgrade state for it
pub fn upgrade_controller(room: &Room) -> Option<Box<dyn ScreepState>> {
    for structure in room.find(find::STRUCTURES, None).iter() {
        if let StructureObject::StructureController(controller) = structure {
            return Some(Box::new(UpgradeState::new(controller.id())));
        }
    }
    None
}
