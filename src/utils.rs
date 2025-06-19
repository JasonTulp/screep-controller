// Helpful utility functions for the project.
use screeps::{
    find, ConstructionSite, Creep, FindConstant, HasId, HasPosition, ObjectId, ResourceType, Room,
    StructureObject,
};

pub mod prelude {
    pub use {
        super::find_nearest_construction_site, super::find_nearest_object,
        super::get_total_upgrade_energy,
    };
}

// Gets the nearest object based on distance from the creep
pub fn find_nearest_object<R>(
    creep: &Creep,
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
        .min_by_key(|obj| creep.pos().get_range_to(obj.pos()))?;

    Some(nearest.clone().id())
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
// 
// // Gets count of creeps in a room with a specific state
// pub fn get_count_of_creeps_with_state(room: &Room, state_name: &str) -> usize {
//     room.find(find::CREEPS, None)
//         .iter()
//         .filter(|creep| {
//             creep
//                 .memory()
//                 .get("state")
//                 .map_or(false, |s| s == state_name)
//         })
//         .count()
// }
