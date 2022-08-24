#[derive(Clone)]
pub struct Child {
    parent: hecs::Entity,
    next_sibling: hecs::Entity,
    previous_sibling: hecs::Entity,
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
    fn iterate_children(&self, parent: hecs::Entity) -> ChildIterator;
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

        let mut next_sibling = child;
        let mut previous_sibling = child;

        if let Ok(parent) = self.get::<&Parent>(parent) {
            if let Some(arbitrary_child) = parent.arbitrary_child {
                let mut arbitrary_child_component =
                    self.get::<&mut Child>(arbitrary_child).unwrap();
                next_sibling = arbitrary_child_component.next_sibling;
                arbitrary_child_component.next_sibling = child;
                previous_sibling = arbitrary_child;
            }
        }
        let _ = self.insert_one(
            parent,
            Parent {
                arbitrary_child: Some(child),
            },
        );
        let _ = self.insert_one(
            child,
            Child {
                parent,
                next_sibling,
                previous_sibling,
            },
        );

        if let Ok(mut c) = self.get::<&mut Child>(next_sibling) {
            c.previous_sibling = next_sibling;
        }

        Ok(())
    }

    fn unparent(&mut self, child_entity: hecs::Entity) -> Result<(), hecs::NoSuchEntity> {
        // If there's currently a parent ensure it still points to a valid child.
        let mut previous_and_next_sibling = None;
        if let Ok(child) = self.get::<&Child>(child_entity) {
            if let Ok(mut parent) = self.get::<&mut Parent>(child.parent) {
                if parent.arbitrary_child == Some(child_entity) {
                    parent.arbitrary_child = Some(child.next_sibling);
                }
            }
            previous_and_next_sibling = Some((child.previous_sibling, child.next_sibling));
        }

        // Connect siblings
        if let Some((previous, next)) = previous_and_next_sibling {
            self.get::<&mut Child>(previous).unwrap().next_sibling = next;
            self.get::<&mut Child>(next).unwrap().previous_sibling = previous;
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

    fn iterate_children(&self, parent: hecs::Entity) -> ChildIterator {
        let next_child = self
            .get::<&Parent>(parent)
            .map(|p| p.arbitrary_child)
            .ok()
            .flatten();
        ChildIterator {
            world: self,
            next_child,
            start: next_child,
        }
    }
}

pub struct ChildIterator<'a> {
    world: &'a hecs::World,
    next_child: Option<hecs::Entity>,
    start: Option<hecs::Entity>,
}

impl<'a> Iterator for ChildIterator<'a> {
    type Item = hecs::Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.next_child.take();
        if let Some(result) = result {
            if let Ok(child) = self.world.get::<&Child>(result) {
                self.next_child = Some(child.next_sibling);
                if self.next_child == self.start {
                    self.next_child = None;
                }
            }
        }
        result
    }
}
