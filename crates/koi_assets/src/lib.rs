use koi_slotmap::SlotMapHandle;

pub struct AssetStore<Asset> {
    slot_map: koi_slotmap::SlotMap<Asset>,
    path_to_slotmap: std::collections::HashMap<String, koi_slotmap::SlotMapHandle<Asset>>,
    pub need_loading: Vec<(String, Handle<Asset>)>,
}

impl<Asset> AssetStore<Asset> {
    pub fn new() -> Self {
        Self {
            slot_map: koi_slotmap::SlotMap::new(),
            path_to_slotmap: std::collections::HashMap::new(),
            need_loading: Vec::new(),
        }
    }

    pub fn items_iter(&self) -> std::slice::Iter<Asset> {
        self.slot_map.items_iter()
    }

    pub fn add(&mut self, asset: Asset) -> Handle<Asset> {
        Handle {
            slot_map_handle: self.slot_map.push(asset),
            phantom: std::marker::PhantomData,
        }
    }

    /// Used to initialize static variables
    /// Adds an asset and leaks it.
    /// Panics of `handle_to_check` is not equivalent to the handle allocated.
    pub fn add_and_leak(&mut self, asset: Asset, handle_to_check: &Handle<Asset>) {
        let handle = self.add(asset);
        assert_eq!(handle, *handle_to_check);
    }

    pub fn get(&self, handle: &Handle<Asset>) -> &Asset {
        self.slot_map.get(&handle.slot_map_handle).unwrap()
    }

    pub fn get_mut(&mut self, handle: &Handle<Asset>) -> &mut Asset {
        self.slot_map.get_mut(&handle.slot_map_handle).unwrap()
    }
}

impl<Asset: Loadable> AssetStore<Asset> {
    pub fn load(&mut self, path: &str) -> Handle<Asset> {
        if let Some(slot_map_handle) = self.path_to_slotmap.get(path) {
            Handle {
                slot_map_handle: slot_map_handle.clone(),
                phantom: std::marker::PhantomData,
            }
        } else {
            // For some things it might be a bit much to construct a default value each time.
            let handle = self.add(Asset::default());
            self.need_loading.push((path.into(), handle.clone()));
            handle
        }
    }
}

#[derive(Copy)]
pub struct Handle<T> {
    slot_map_handle: SlotMapHandle<T>,
    phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> core::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Handle")
            .field("index", &self.slot_map_handle.index())
            .finish()
    }
}

impl<T> Handle<T> {
    /// Construct a handle directly from an underlying index.
    /// This is used internally to set up global asset constants.
    pub const fn from_index(index: usize) -> Self {
        Self {
            slot_map_handle: SlotMapHandle::from_index(index),
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            slot_map_handle: self.slot_map_handle.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.slot_map_handle == other.slot_map_handle
    }
}

impl<T> Eq for Handle<T> {}

impl<T> PartialOrd for Handle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Handle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.slot_map_handle.cmp(&other.slot_map_handle)
    }
}

pub trait Loadable: Default {}
