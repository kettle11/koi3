use koi_hierarchy::*;

pub use hecs::*;

pub struct World(hecs::World);

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self(hecs::World::new())
    }

    pub fn despawn(&mut self, entity: hecs::Entity) -> Result<(), hecs::NoSuchEntity> {
        self.0.despawn_hierarchy(entity)
    }
}

impl core::ops::Deref for World {
    type Target = hecs::World;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for World {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
