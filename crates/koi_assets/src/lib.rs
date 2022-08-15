mod slotmap;
use slotmap::*;

pub struct AssetStore<Asset> {
    slot_map: SlotMap<AssetEntry<Asset>>,
    path_to_slotmap: std::collections::HashMap<String, WeakHandle<Asset>>,
    drop_channel_sender: std::sync::mpsc::Sender<usize>,
    drop_channel_receiver: std::sync::mpsc::Receiver<usize>,
    pub need_loading: Vec<(String, Handle<Asset>)>,
}

struct AssetEntry<Asset> {
    asset: Asset,
    path: Option<String>,
}

impl<Asset> AssetStore<Asset> {
    pub fn new(placeholder: Asset) -> Self {
        let (drop_channel_sender, drop_channel_receiver) = std::sync::mpsc::channel();
        Self {
            slot_map: SlotMap::new(AssetEntry {
                asset: placeholder,
                path: None,
            }),
            path_to_slotmap: std::collections::HashMap::new(),
            need_loading: Vec::new(),
            drop_channel_receiver,
            drop_channel_sender,
        }
    }

    /// Removes all assets without a handle pointing to them.
    pub fn cleanup_dropped_assets(&mut self) {
        for _ in self.get_dropped_assets() {}
    }

    /// Iterates over and removes all dropped assets without a handle pointing to them.
    pub fn get_dropped_assets(&mut self) -> impl Iterator<Item = Asset> + '_ {
        self.drop_channel_receiver
            .try_iter()
            .map(|indirection_index| {
                // Todo: Also remove from path_to_slotmap if necessary.
                println!("DROPPING ASSET: {:?}", std::any::type_name::<Asset>());
                let AssetEntry { path, asset } = self
                    .slot_map
                    .remove(SlotMapHandle::from_index(indirection_index));

                if let Some(path) = path {
                    self.path_to_slotmap.remove(&path);
                }
                asset
            })
    }

    pub fn items_iter(&self) -> impl Iterator<Item = &Asset> + '_ {
        self.slot_map.items_iter().map(|a| &a.asset)
    }

    fn new_handle(&mut self, slot_map_handle: SlotMapHandle<AssetEntry<Asset>>) -> Handle<Asset> {
        Handle {
            drop_handle: Some(std::sync::Arc::new(DropHandle {
                indirection_index: slot_map_handle.index(),
                channel: SyncGuard::new(self.drop_channel_sender.clone()),
            })),
            slot_map_handle,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn add(&mut self, asset: Asset) -> Handle<Asset> {
        let slot_map_handle = self.slot_map.push(AssetEntry { asset, path: None });
        self.new_handle(slot_map_handle)
    }

    /// Used to initialize static variables
    /// Adds an asset and leaks it.
    /// Panics of `handle_to_check` is not equivalent to the handle allocated.
    pub fn add_and_leak(&mut self, asset: Asset, handle_to_check: &Handle<Asset>) {
        let handle = self.add(asset);
        assert_eq!(handle, *handle_to_check);
        std::mem::forget(handle);
    }

    pub fn get(&self, handle: &Handle<Asset>) -> &Asset {
        &self.slot_map.get(&handle.slot_map_handle).unwrap().asset
    }

    pub fn get_mut(&mut self, handle: &Handle<Asset>) -> &mut Asset {
        &mut self
            .slot_map
            .get_mut(&handle.slot_map_handle)
            .unwrap()
            .asset
    }

    pub fn reload(&mut self) {
        for (name, weak_handle) in &mut self.path_to_slotmap {
            if let Some(handle) = weak_handle.upgrade() {
                self.need_loading.push((name.clone(), handle));
            }
        }
    }
}

impl<Asset: Loadable> AssetStore<Asset> {
    pub fn load(&mut self, path: &str) -> Handle<Asset> {
        if let Some(weak_handle) = self
            .path_to_slotmap
            .get(path)
            .and_then(|weak_handle| weak_handle.upgrade())
        {
            weak_handle
        } else {
            let slot_map_handle = self.slot_map.new_handle_pointing_at_placeholder();
            let handle = self.new_handle(slot_map_handle);
            self.need_loading.push((path.into(), handle.clone()));
            handle
        }
    }
}

pub struct Handle<Asset> {
    slot_map_handle: SlotMapHandle<AssetEntry<Asset>>,
    drop_handle: Option<std::sync::Arc<DropHandle>>,
    phantom: std::marker::PhantomData<fn() -> Asset>,
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
            drop_handle: None,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self {
            slot_map_handle: self.slot_map_handle.clone(),
            drop_handle: self.drop_handle.clone(),
            phantom: self.phantom,
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

pub struct DropHandle {
    indirection_index: usize,
    channel: SyncGuard<std::sync::mpsc::Sender<usize>>,
}

impl Drop for DropHandle {
    fn drop(&mut self) {
        let _ = self.channel.inner().send(self.indirection_index);
    }
}

pub trait Loadable {}

pub struct SyncGuard<T> {
    inner: T,
}
impl<T> SyncGuard<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
    pub fn inner(&mut self) -> &mut T {
        &mut self.inner
    }
}

/// # Safety
/// Nobody in the Rust Gamedev Discord yelled at me about this.
unsafe impl<T> Sync for SyncGuard<T> {}

struct WeakHandle<Asset> {
    inner_handle: Handle<Asset>,
    drop_handle: std::sync::Weak<DropHandle>,
}

impl<Asset> WeakHandle<Asset> {
    /// Upgrades this [WeakHandle<T>] to a full [Handle<T>]
    /// This will return [None] if all [Handle<T>]s have already been dropped.
    pub fn upgrade(&self) -> Option<Handle<Asset>> {
        let mut handle = self.inner_handle.clone();
        handle.drop_handle = Some(self.drop_handle.upgrade()?);
        Some(handle)
    }
}
