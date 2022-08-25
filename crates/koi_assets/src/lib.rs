/// TODO: Paths aren't handled correctly yet.
mod slotmap;
use koi_ecs::Component;
use koi_resources::Resources;
use loader::AssetLoaderTrait;
use slotmap::*;

pub mod loader;

pub struct AssetStoreInner<Asset: AssetTrait> {
    slot_map: SlotMap<Asset>,
    path_to_slotmap: std::collections::HashMap<String, (WeakHandle<Asset>, Asset::Settings)>,
    drop_channel_sender: std::sync::mpsc::Sender<usize>,
    drop_channel_receiver: std::sync::mpsc::Receiver<usize>,
}

impl<Asset: AssetTrait> AssetStoreInner<Asset> {
    pub fn new(placeholder: Asset) -> Self {
        let (drop_channel_sender, drop_channel_receiver) = std::sync::mpsc::channel();
        Self {
            slot_map: SlotMap::new(placeholder),
            path_to_slotmap: std::collections::HashMap::new(),
            drop_channel_receiver,
            drop_channel_sender,
        }
    }

    pub fn cleanup_dropped_assets(&mut self) {
        for _ in self.drop_channel_receiver.try_iter() {}
    }

    /// Iterates over and removes all dropped assets without a handle pointing to them.
    pub fn get_dropped_assets(&mut self) -> impl Iterator<Item = Asset> + '_ {
        self.drop_channel_receiver
            .try_iter()
            .filter_map(|indirection_index| {
                let slot_map_handle = SlotMapHandle::from_index(indirection_index);
                if !self.slot_map.handle_is_placeholder(&slot_map_handle) {
                    // Todo: Also remove from path_to_slotmap if necessary.
                    println!("DROPPING ASSET: {:?}", std::any::type_name::<Asset>());
                    let (asset, path) = self.slot_map.remove(slot_map_handle);

                    if let Some(path) = path {
                        self.path_to_slotmap.remove(&path);
                    }
                    Some(asset)
                } else {
                    None
                }
            })
    }

    pub fn items_iter(&self) -> impl Iterator<Item = &Asset> + '_ {
        self.slot_map.items_iter()
    }

    fn new_handle(&mut self, slot_map_handle: SlotMapHandle<Asset>) -> Handle<Asset> {
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
        let slot_map_handle = self.slot_map.push(asset, None);
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

    // pub fn replace_placeholder(&mut self, handle: &Handle<Asset>, asset: Asset) {
    //     self.slot_map
    //         .replace_placeholder(&handle.slot_map_handle, asset)
    // }

    pub fn get(&self, handle: &Handle<Asset>) -> &Asset {
        &self.slot_map.get(&handle.slot_map_handle).unwrap()
    }

    pub fn get_mut(&mut self, handle: &Handle<Asset>) -> &mut Asset {
        self.slot_map.get_mut(&handle.slot_map_handle).unwrap()
    }

    pub fn replace(&mut self, handle: &Handle<Asset>, asset: Asset) {
        if self.slot_map.handle_is_placeholder(&handle.slot_map_handle) {
            self.slot_map
                .replace_placeholder(&handle.slot_map_handle, asset);
        } else {
            *self.get_mut(handle) = asset;
        }
    }
}

pub struct AssetStore<Asset: AssetTrait> {
    asset_store_inner: AssetStoreInner<Asset>,
    loader: Box<dyn AssetLoaderTrait<Asset>>,
}

impl<Asset: AssetTrait> AssetStore<Asset> {
    pub fn new(placeholder: Asset) -> Self {
        Self {
            asset_store_inner: AssetStoreInner::new(placeholder),
            loader: Box::new(crate::loader::DoNothingLoader),
        }
    }

    pub fn new_with_load_functions<
        LoadResult: Send + 'static,
        F: std::future::Future<Output = Option<LoadResult>> + Send + 'static,
    >(
        placeholder: Asset,
        load_task: fn(String, Asset::Settings) -> F,
        handle_result: fn(LoadResult, Asset::Settings, &koi_resources::Resources) -> Option<Asset>,
    ) -> Self {
        Self {
            asset_store_inner: AssetStoreInner::new(placeholder),
            loader: Box::new(crate::loader::Loader::new(load_task, handle_result)),
        }
    }

    /// Removes all assets without a handle pointing to them.
    pub fn cleanup_dropped_assets(&mut self) {
        self.asset_store_inner.cleanup_dropped_assets();
    }

    /// Iterates over and removes all dropped assets without a handle pointing to them.
    pub fn get_dropped_assets(&mut self) -> impl Iterator<Item = Asset> + '_ {
        self.asset_store_inner.get_dropped_assets()
    }

    pub fn items_iter(&self) -> impl Iterator<Item = &Asset> + '_ {
        self.asset_store_inner.items_iter()
    }

    pub fn add(&mut self, asset: Asset) -> Handle<Asset> {
        self.asset_store_inner.add(asset)
    }

    /// Used to initialize static variables
    /// Adds an asset and leaks it.
    /// Panics of `handle_to_check` is not equivalent to the handle allocated.
    pub fn add_and_leak(&mut self, asset: Asset, handle_to_check: &Handle<Asset>) {
        self.asset_store_inner.add_and_leak(asset, handle_to_check)
    }

    // pub fn replace_placeholder(&mut self, handle: &Handle<Asset>, asset: Asset) {
    //     self.slot_map
    //         .replace_placeholder(&handle.slot_map_handle, asset)
    // }

    pub fn get(&self, handle: &Handle<Asset>) -> &Asset {
        self.asset_store_inner.get(handle)
    }

    pub fn get_mut(&mut self, handle: &Handle<Asset>) -> &mut Asset {
        self.asset_store_inner.get_mut(handle)
    }

    pub fn replace(&mut self, handle: &Handle<Asset>, asset: Asset) {
        self.asset_store_inner.replace(handle, asset)
    }

    pub fn finalize_asset_loads(&mut self, resources: &Resources) {
        let AssetStore {
            asset_store_inner,
            loader,
        } = self;

        loader.finalize_load_on_main_thread(resources, asset_store_inner);
    }

    pub fn load(&mut self, path: &str, settings: Asset::Settings) -> Handle<Asset> {
        if let Some(weak_handle) = self
            .asset_store_inner
            .path_to_slotmap
            .get(path)
            .and_then(|weak_handle| weak_handle.0.upgrade())
        {
            weak_handle
        } else {
            let slot_map_handle = self
                .asset_store_inner
                .slot_map
                .new_handle_pointing_at_placeholder(Some(path.into()));
            let handle = self.asset_store_inner.new_handle(slot_map_handle);
            self.asset_store_inner
                .path_to_slotmap
                .insert(path.into(), (handle.to_weak(), settings.clone()));
            self.loader.load(path.into(), settings, handle.clone());
            handle
        }
    }

    /// Reloads all assets that were loaded from a path.
    pub fn reload(&mut self) {
        for (path, weak_handle) in &self.asset_store_inner.path_to_slotmap {
            if let Some(handle) = weak_handle.0.upgrade() {
                self.loader
                    .load(path.into(), weak_handle.1.clone(), handle.clone());
            }
        }
    }

    /// How many assets are currently loading.
    pub fn currently_loading(&self) -> usize {
        self.loader.currently_loading()
    }
}

#[derive(Component)]
pub struct Handle<Asset: 'static> {
    slot_map_handle: SlotMapHandle<Asset>,
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
    pub const PLACEHOLDER: Self = Handle {
        slot_map_handle: SlotMapHandle::from_index(0),
        drop_handle: None,
        phantom: std::marker::PhantomData,
    };

    /// Construct a handle directly from an underlying index.
    /// This is used internally to set up global asset constants.
    pub const fn from_index(index: usize) -> Self {
        Self {
            slot_map_handle: SlotMapHandle::from_index(index),
            drop_handle: None,
            phantom: std::marker::PhantomData,
        }
    }

    fn to_weak(&self) -> WeakHandle<T> {
        WeakHandle {
            inner_handle: Handle {
                slot_map_handle: self.slot_map_handle.clone(),
                drop_handle: None,
                phantom: std::marker::PhantomData,
            },
            drop_handle: std::sync::Arc::downgrade(&self.drop_handle.as_ref().unwrap()),
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

pub trait AssetTrait: Sized + 'static {
    type Settings: Clone + Send;
}

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

struct WeakHandle<Asset: 'static> {
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
