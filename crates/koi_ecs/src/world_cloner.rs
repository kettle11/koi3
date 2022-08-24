use crate::World;
use hecs::*;
use std::any::TypeId;

pub trait WorldClonableTrait: Sized + Sync + Send + 'static {
    fn clone_with_context(&self, entity_migrator: &EntityMigrator) -> Self;
}

pub struct WorldCloner {
    cloners: std::collections::HashMap<TypeId, RegisteredComponent>,
}

struct RegisteredComponent {
    add_type: fn(&mut ColumnBatchType),
    clone_components: fn(&Archetype, &mut ColumnBatchBuilder, &EntityMigrator),
}

pub struct EntityMigrator<'a> {
    old_to_new_entities: &'a [Option<Entity>],
}

impl<'a> EntityMigrator<'a> {
    pub fn migrate(&self, old_entity: Entity) -> Option<Entity> {
        self.old_to_new_entities
            .get(old_entity.id() as usize)
            .cloned()
            .flatten()
    }
}

impl WorldCloner {
    pub fn new() -> Self {
        Self {
            cloners: std::collections::HashMap::new(),
        }
    }

    pub fn register_clone_type<T: WorldClonableTrait>(&mut self) {
        self.cloners.insert(
            std::any::TypeId::of::<T>(),
            RegisteredComponent {
                add_type: |column_batch_type| {
                    column_batch_type.add::<T>();
                },
                clone_components: |archetype, column_batch_builder, entity_migrator| {
                    let column = archetype.get::<&mut T>().unwrap();
                    let mut writer = column_batch_builder.writer().unwrap();
                    for c in column.iter() {
                        let _ = writer.push(c.clone_with_context(entity_migrator));
                    }
                },
            },
        );
    }

    pub fn clone_world(&self, source_world: &mut World, destination_world: &mut World) {
        let mut reserved_entities = destination_world.reserve_entities(source_world.len());

        let mut old_to_new_entity = vec![None; source_world.len() as usize];
        let mut old_to_new_temp = Vec::new();

        for entity in source_world.iter() {
            let index = entity.entity().id() as usize;
            old_to_new_entity.resize(index.max(old_to_new_entity.len()), None);
            old_to_new_entity[index] = Some(reserved_entities.next().unwrap());
        }

        destination_world.flush();

        let entity_migrator = EntityMigrator {
            old_to_new_entities: &old_to_new_entity,
        };

        for archetype in source_world.archetypes() {
            let mut column_batch_type = ColumnBatchType::new();
            for type_id in archetype.component_types() {
                if let Some(cloner) = self.cloners.get(&type_id) {
                    (cloner.add_type)(&mut column_batch_type);
                } else {
                    println!("WARNING: Component is uncloned because it is not registered with WorldCloner. Type ID: {:?}", type_id);
                }
            }

            old_to_new_temp.clear();
            old_to_new_temp.reserve(archetype.len() as usize);
            for entity in archetype.ids() {
                old_to_new_temp.push(old_to_new_entity[*entity as usize].unwrap());
            }

            let mut column_batch_builder =
                ColumnBatchBuilder::new(column_batch_type, archetype.len());
            for type_id in archetype.component_types() {
                if let Some(cloner) = self.cloners.get(&type_id) {
                    (cloner.clone_components)(
                        archetype,
                        &mut column_batch_builder,
                        &entity_migrator,
                    );
                }
            }

            let column_batch = column_batch_builder.build().unwrap();
            destination_world.spawn_column_batch_at(&old_to_new_temp, column_batch);
        }
    }
}

#[test]
fn clone_world_test() {
    let mut world_cloner = WorldCloner::new();

    #[derive(Clone)]
    struct A;

    #[derive(Clone)]
    struct B;

    impl WorldClonableTrait for A {
        fn clone_with_context(&self, _: &EntityMigrator) -> Self {
            self.clone()
        }
    }
    world_cloner.register_clone_type::<A>();

    let mut world_a = World::new();
    world_a.spawn((A,));
    world_a.spawn((A, B));
    world_a.spawn((B,));

    let mut world_b = World::new();
    world_cloner.clone_world(&mut world_a, &mut world_b);

    assert_eq!(world_b.len(), world_a.len());
}
