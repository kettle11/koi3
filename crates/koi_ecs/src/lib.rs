pub use hierarchy::*;

pub mod world_cloner;

pub use hecs::*;
pub use koi_ecs_derive::*;
pub use world_cloner::{EntityMigrator, WorldClonableTrait};

mod hierarchy;
pub use hierarchy::*;

pub struct World {
    hecs_world: hecs::World,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            hecs_world: hecs::World::new(),
        }
    }

    pub fn despawn(&mut self, entity: hecs::Entity) -> Result<(), hecs::NoSuchEntity> {
        self.hecs_world.despawn_hierarchy(entity)
    }
}

impl Clone for World {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl core::ops::Deref for World {
    type Target = hecs::World;

    fn deref(&self) -> &Self::Target {
        &self.hecs_world
    }
}

impl core::ops::DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hecs_world
    }
}

pub trait WorldExtensions {
    fn enable_component<T: Sync + Send + 'static>(&mut self, entity: hecs::Entity);
    fn disable_component<T: Sync + Send + 'static>(&mut self, entity: hecs::Entity);
}

struct Disabled<T>(T);

impl WorldExtensions for hecs::World {
    fn enable_component<T: Sync + Send + 'static>(&mut self, entity: hecs::Entity) {
        if let Ok(t) = self.remove::<(Disabled<T>,)>(entity) {
            self.insert(entity, t).unwrap();
        }
    }
    fn disable_component<T: Sync + Send + 'static>(&mut self, entity: hecs::Entity) {
        if let Ok(t) = self.remove::<(T,)>(entity) {
            self.insert(entity, (Disabled(t),)).unwrap();
        }
    }
}
