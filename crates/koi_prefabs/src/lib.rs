use koi_ecs::HierachyExtension;
use koi_resources::*;

#[cfg(feature = "gltf")]
mod gltf;

pub struct Prefab(pub koi_ecs::World);

impl Prefab {
    // Spawns all the top level entities in the prefab parented to a parent.
    pub fn spawn_with_transform(
        &mut self,
        destination: &mut koi_ecs::World,
        cloner: &mut koi_ecs::world_cloner::WorldCloner,
        parent_transform: koi_transform::Transform,
    ) {
        let root_node = destination.spawn((parent_transform,));
        self.spawn_with_parent(destination, cloner, root_node);
    }

    pub fn spawn_with_parent(
        &mut self,
        destination: &mut koi_ecs::World,
        cloner: &mut koi_ecs::world_cloner::WorldCloner,
        parent_entity: koi_ecs::Entity,
    ) {
        // TODO: Avoid this allocation
        let mut top_level_entities = Vec::new();
        for (e, _) in self
            .0
            .query::<koi_ecs::Without<(), &koi_ecs::Child>>()
            .iter()
        {
            top_level_entities.push(e);
        }

        let migrator = cloner.clone_world(&mut self.0, destination);

        for e in top_level_entities {
            let e = migrator.migrate(e).unwrap();
            destination.set_parent(parent_entity, e).unwrap();
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
        .expect("Expected file extension for prefab")
        .to_lowercase();

    match &*extension {
        #[cfg(feature = "gltf")]
        "gltf" => gltf::load_gltf(path).await,
        #[cfg(feature = "gltf")]
        "glb" => gltf::load_glb(path).await,
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

    let mut worlds_to_add = Vec::new();
    resources
        .get_mut::<koi_events::EventHandlers>()
        .add_universal_handler(move |_event, world, resources| {
            let mut prefabs = resources.get::<koi_assets::AssetStore<Prefab>>();
            prefabs.finalize_asset_loads(resources);

            // Delayed spawning of prefabs as they load.
            {
                for (entity, (_transform, prefab)) in world
                    .query::<(&koi_transform::Transform, &koi_assets::Handle<Prefab>)>()
                    .iter()
                {
                    if !prefabs.is_placeholder(prefab) {
                        worlds_to_add.push(entity);
                    }
                }

                let mut world_cloner = resources.get::<koi_ecs::WorldCloner>();

                for entity in worlds_to_add.drain(..) {
                    let handle = world
                        .remove_one::<koi_assets::Handle<Prefab>>(entity)
                        .unwrap();
                    let prefab = prefabs.get_mut(&handle);
                    prefab.spawn_with_parent(world, &mut world_cloner, entity);
                }
            }
        });
}

enum PrefabLoadResult {
    #[cfg(feature = "gltf")]
    GlTf(crate::gltf::GlTfLoadResult),
}
