use koi_resources::*;

#[cfg(feature = "gltf")]
mod gltf;

pub struct Prefab(pub koi_ecs::World);

impl Prefab {
    // Transforms all the top level [koi_ecs::Entity]s by `transform`.
    pub fn spawn_with_transform(
        &mut self,
        destination: &mut koi_ecs::World,
        cloner: &mut koi_ecs::world_cloner::WorldCloner,
        parent_transform: koi_transform::Transform,
    ) {
        // TODO: Avoid this allocation
        let mut top_level_entities = Vec::new();
        // This modifies the prefab, that's probably bad.
        for (e, _) in self
            .0
            .query::<koi_ecs::Without<&mut koi_transform::Transform, &koi_ecs::Child>>()
            .iter()
        {
            top_level_entities.push(e);
        }

        let migrator = cloner.clone_world(&mut self.0, destination);
        let parent_transform = parent_transform.local_to_world();

        for e in top_level_entities {
            let e = migrator.migrate(e).unwrap();
            let mut transform = destination.get::<&mut koi_transform::Transform>(e).unwrap();
            let mat = transform.local_to_world();
            *transform = koi_transform::Transform::from_mat4(parent_transform * mat);
        }
    }
}

impl koi_assets::AssetTrait for Prefab {
    type Settings = ();
}

async fn load_world(path: String, _settings: ()) -> Option<PrefabLoadResult> {
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
        |result, _settings, resources| match result {
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
