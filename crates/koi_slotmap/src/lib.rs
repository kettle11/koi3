pub struct SlotMap<T> {
    items: Vec<T>,
    item_to_indirection_index: Vec<usize>,
    indirection_indices: Vec<usize>,
    free_indirection_indices: Vec<usize>,
}

#[derive(Copy)]
pub struct SlotMapHandle<T> {
    indirection_index: usize,
    phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T> SlotMapHandle<T> {
    pub const fn from_index(index: usize) -> Self {
        Self {
            indirection_index: index,
            phantom: std::marker::PhantomData,
        }
    }

    pub const fn index(&self) -> usize {
        self.indirection_index
    }
}

impl<T> PartialEq for SlotMapHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.indirection_index == other.indirection_index
    }
}

impl<T> Eq for SlotMapHandle<T> {}

impl<T> PartialOrd for SlotMapHandle<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for SlotMapHandle<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.indirection_index.cmp(&other.indirection_index)
    }
}

impl<T> Clone for SlotMapHandle<T> {
    fn clone(&self) -> Self {
        Self {
            indirection_index: self.indirection_index,
            phantom: self.phantom,
        }
    }
}

impl<T> Default for SlotMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SlotMap<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            indirection_indices: Vec::new(),
            item_to_indirection_index: Vec::new(),
            free_indirection_indices: Vec::new(),
        }
    }

    pub fn items_iter(&self) -> std::slice::Iter<T> {
        self.items.iter()
    }

    pub fn push(&mut self, item: T) -> SlotMapHandle<T> {
        let items_index = self.items.len();
        self.items.push(item);

        let indirection_index = if let Some(indirection_index) = self.free_indirection_indices.pop()
        {
            self.indirection_indices[indirection_index] = items_index;
            indirection_index
        } else {
            let indirection_index = self.indirection_indices.len();
            self.indirection_indices.push(items_index);
            indirection_index
        };
        self.item_to_indirection_index.push(indirection_index);

        SlotMapHandle {
            indirection_index,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn remove(&mut self, handle: SlotMapHandle<T>) -> T {
        let item_index = self.indirection_indices[handle.indirection_index];
        self.indirection_indices[*self.item_to_indirection_index.last().unwrap()] = item_index;
        let removed_item = self.items.swap_remove(item_index);
        self.item_to_indirection_index.swap_remove(item_index);
        self.free_indirection_indices.push(handle.indirection_index);
        removed_item
    }

    pub fn get(&self, handle: &SlotMapHandle<T>) -> Option<&T> {
        self.items
            .get(self.indirection_indices[handle.indirection_index])
    }

    pub fn get_mut(&mut self, handle: &SlotMapHandle<T>) -> Option<&mut T> {
        self.items
            .get_mut(self.indirection_indices[handle.indirection_index])
    }
}
