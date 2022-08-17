pub struct Resources {
    resources: std::collections::HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl Default for Resources {
    fn default() -> Self {
        Self::new()
    }
}

impl Resources {
    pub fn new() -> Self {
        Self {
            resources: std::collections::HashMap::new(),
        }
    }

    #[inline]
    pub fn add<T: 'static>(&mut self, resource: T) {
        self.resources.insert(
            std::any::TypeId::of::<T>(),
            Box::new(std::sync::RwLock::new(resource)),
        );
    }

    #[inline]
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.resources
            .remove(&std::any::TypeId::of::<T>())?
            .downcast::<std::sync::RwLock<T>>()
            .unwrap()
            .into_inner()
            .ok()
    }

    #[inline]
    pub fn get<T: 'static>(&self) -> std::sync::RwLockWriteGuard<T> {
        self.try_get::<T>().unwrap()
    }

    #[inline]
    pub fn get_mut<T: 'static>(&mut self) -> &mut T {
        // This could use downcast-mut unchecked in the future.
        self.resources
            .get_mut(&std::any::TypeId::of::<T>())
            .unwrap()
            .downcast_mut::<std::sync::RwLock<T>>()
            .unwrap()
            .get_mut()
            .unwrap()
    }

    #[inline]
    pub fn try_get<T: 'static>(&self) -> Option<std::sync::RwLockWriteGuard<T>> {
        Some(
            self.resources
                .get(&std::any::TypeId::of::<T>())?
                .downcast_ref::<std::sync::RwLock<T>>()
                .unwrap()
                .try_write()
                .unwrap(),
        )
    }
}
