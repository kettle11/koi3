use koi_resources::*;

#[cfg(feature = "gltf")]
mod gltf;

pub struct Prefab(pub koi_ecs::World);

impl koi_assets::AssetTrait for Prefab {
    type Settings = ();
}

async fn load_world(path: String, settings: ()) -> Option<PrefabLoadResult> {
    let extension = std::path::Path::new(&path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .expect("Expected image file extension")
        .to_lowercase();

    match &*extension {
        #[cfg(feature = "gltf")]
        "gltf" => gltf::load_gltf(path).await,
        _ => {
            println!(
                "Error loading prefab. Unsupported file extension: {extension} for path {path}"
            );
            None
        }
    }
}

pub fn initialize_plugin(resources: &mut Resources) {
    let worlds = koi_assets::AssetStore::new_with_load_functions(
        Prefab(koi_ecs::World::new()),
        load_world,
        |result, settings, resources| match result {
            PrefabLoadResult::GlTf(gltf_load_result) => {
                gltf::finalize_gltf_load(resources, gltf_load_result)
            }
        },
    );
    resources.add(worlds);

    resources
        .get_mut::<koi_events::EventHandlers>()
        .add_universal_handler(|_event, _world, resources| {
            let mut prefabs = resources.get::<koi_assets::AssetStore<Prefab>>();
            prefabs.finalize_asset_loads(resources);
        });
}

enum PrefabLoadResult {
    #[cfg(feature = "gltf")]
    GlTf(crate::gltf::GlTfLoadResult),
}
