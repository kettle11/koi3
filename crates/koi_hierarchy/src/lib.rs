#[derive(Clone)]
pub struct Child {
    parent: Option<hecs::Entity>,
    next_sibling: hecs::Entity,
}

#[derive(Clone)]
pub struct Parent {
    arbitrary_child: Option<hecs::Entity>,
}

pub trait HierachyExtension {
    fn set_parent(
        &mut self,
        parent: hecs::Entity,
        child: hecs::Entity,
    ) -> Result<(), hecs::NoSuchEntity>;

    fn unparent(&mut self, child: hecs::Entity) -> Result<(), hecs::NoSuchEntity>;
    fn despawn_hierarchy(&mut self, parent: hecs::Entity) -> Result<(), hecs::NoSuchEntity>;
}

impl HierachyExtension for hecs::World {
    fn set_parent(
        &mut self,
        parent: hecs::Entity,
        child: hecs::Entity,
    ) -> Result<(), hecs::NoSuchEntity> {
        if !self.contains(child) {
            return Err(hecs::NoSuchEntity);
        }

        let _ = self.insert_one(
            parent,
            Parent {
                arbitrary_child: Some(child),
            },
        );

        // TODO!
        // Insert next sibling here.

        Ok(())
    }

    fn unparent(&mut self, child_entity: hecs::Entity) -> Result<(), hecs::NoSuchEntity> {
        // If there's currently a parent ensure it still points to a valid child.
        if let Ok(child) = self.get::<&Child>(child_entity) {
            if let Some(parent) = child.parent {
                if let Ok(mut parent) = self.get::<&mut Parent>(parent) {
                    if parent.arbitrary_child == Some(child_entity) {
                        parent.arbitrary_child = Some(child.next_sibling);
                    }
                }
            }
        }

        if let Err(hecs::ComponentError::NoSuchEntity) = self.remove_one::<Child>(child_entity) {
            return Err(hecs::NoSuchEntity);
        }
        Ok(())
    }

    fn despawn_hierarchy(&mut self, parent: hecs::Entity) -> Result<(), hecs::NoSuchEntity> {
        // Update the parent
        self.unparent(parent)?;
        self.despawn(parent)?;

        // Despawn all children and their siblings recursively.
        if let Ok(hierarchy_node) = self.get::<&mut Parent>(parent).map(|h| h.clone()) {
            if let Some(start_child) = hierarchy_node.arbitrary_child {
                let mut current_child = start_child;
                loop {
                    self.despawn_hierarchy(current_child)?;
                    current_child = self.get::<&Child>(current_child).unwrap().next_sibling;
                    if start_child == current_child {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
