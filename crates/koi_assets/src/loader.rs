use crate::{AssetStoreInner, AssetTrait};
use koi_resources::Resources;

pub trait AssetLoaderTrait<Asset: AssetTrait> {
    fn finalize_load_on_main_thread(
        &mut self,
        _resources: &koi_resources::Resources,
        _asset_store_inner: &mut AssetStoreInner<Asset>,
    ) {
    }
    fn load(&mut self, _path: String, _settings: Asset::Settings, _handle: crate::Handle<Asset>) {}
    fn currently_loading(&self) -> usize {
        0
    }
}
/// Abstract away off-thread loading boilerplate.
pub struct Loader<
    LoadResult,
    Asset: AssetTrait,
    F: std::future::Future<Output = Option<LoadResult>> + Send,
> {
    load_task: fn(String, Asset::Settings) -> F,
    handle_result: fn(LoadResult, Asset::Settings, &Resources) -> Option<Asset>,
    sender: std::sync::mpsc::Sender<(LoadResult, Asset::Settings, crate::Handle<Asset>)>,
    receiver: std::sync::mpsc::Receiver<(LoadResult, Asset::Settings, crate::Handle<Asset>)>,
    currently_loading: usize,
}

impl<
        LoadResult: 'static + Send,
        Asset: AssetTrait + 'static,
        F: std::future::Future<Output = Option<LoadResult>> + Send + 'static,
    > Loader<LoadResult, Asset, F>
{
    pub fn new(
        load_task: fn(String, Asset::Settings) -> F,
        handle_result: fn(LoadResult, Asset::Settings, &Resources) -> Option<Asset>,
    ) -> Self {
        let (sender, receiver) = std::sync::mpsc::channel();
        Self {
            load_task,
            handle_result,
            sender,
            receiver,
            currently_loading: 0,
        }
    }

    pub fn load_on_main_thread(
        &mut self,
        resources: &koi_resources::Resources,
        asset_store: &mut crate::AssetStoreInner<Asset>,
    ) {
        while let Ok((load_result, settings, handle)) = self.receiver.try_recv() {
            if let Some(asset) = (self.handle_result)(load_result, settings, resources) {
                asset_store.replace(&handle, asset)
            }
            self.currently_loading -= 1;
        }
    }

    pub fn begin_load(
        &mut self,
        path: String,
        settings: Asset::Settings,
        handle: crate::Handle<Asset>,
    ) {
        let load_task = self.load_task;
        let sender = self.sender.clone();
        self.currently_loading += 1;

        ktasks::spawn(async move {
            let settings0 = settings.clone();
            if let Some(result) = (load_task)(path, settings0).await {
                sender.send((result, settings, handle)).unwrap();
            }
        })
        .run();
    }
}

impl<
        LoadResult: Send + 'static,
        Asset: AssetTrait,
        F: std::future::Future<Output = Option<LoadResult>> + Send + 'static,
    > AssetLoaderTrait<Asset> for Loader<LoadResult, Asset, F>
{
    fn finalize_load_on_main_thread(
        &mut self,
        resources: &koi_resources::Resources,
        asset_store_inner: &mut AssetStoreInner<Asset>,
    ) {
        self.load_on_main_thread(resources, asset_store_inner);
    }
    fn load(&mut self, path: String, settings: Asset::Settings, handle: crate::Handle<Asset>) {
        self.begin_load(path, settings, handle)
    }
    fn currently_loading(&self) -> usize {
        self.currently_loading
    }
}

pub(crate) struct DoNothingLoader;
impl<Asset: AssetTrait> AssetLoaderTrait<Asset> for DoNothingLoader {}
