// A data structure designed to efficiently store data with persistent IDs.
// Slightly tailored to `koi_assets` by special-casing the first slot
// as a default placeholder.
pub struct SlotMap<T> {
    items: Vec<T>,
    item_to_indirection_index: Vec<usize>,
    indirection_indices: Vec<Entry>,
    free_indirection_indices: Vec<usize>,
}

struct Entry {
    item_index: usize,
    path: Option<String>,
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

impl<T> SlotMap<T> {
    pub fn new(placeholder: T) -> Self {
        Self {
            items: vec![placeholder],
            indirection_indices: vec![Entry {
                item_index: 0,
                path: None,
            }],
            item_to_indirection_index: vec![0],
            free_indirection_indices: Vec::new(),
        }
    }

    pub fn items_iter(&self) -> std::slice::Iter<T> {
        self.items.iter()
    }

    fn new_handle_with_index(
        &mut self,
        item_index: usize,
        path: Option<String>,
    ) -> SlotMapHandle<T> {
        let indirection_index = if let Some(indirection_index) = self.free_indirection_indices.pop()
        {
            self.indirection_indices[indirection_index] = Entry { item_index, path };
            indirection_index
        } else {
            let indirection_index = self.indirection_indices.len();
            self.indirection_indices.push(Entry { item_index, path });
            indirection_index
        };
        self.item_to_indirection_index.push(indirection_index);

        SlotMapHandle {
            indirection_index,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn new_handle_pointing_at_placeholder(&mut self, path: Option<String>) -> SlotMapHandle<T> {
        self.new_handle_with_index(0, path)
    }

    pub fn replace_placeholder(&mut self, handle: &SlotMapHandle<T>, item: T) {
        assert!(self.handle_is_placeholder(handle));
        let new_index = self.items.len();
        self.items.push(item);
        self.indirection_indices[handle.indirection_index].item_index = new_index;
    }

    pub fn push(&mut self, item: T, path: Option<String>) -> SlotMapHandle<T> {
        let item_index = self.items.len();
        self.items.push(item);
        self.new_handle_with_index(item_index, path)
    }

    pub fn handle_is_placeholder(&mut self, handle: &SlotMapHandle<T>) -> bool {
        self.indirection_indices[handle.indirection_index].item_index == 0
    }

    pub fn remove(&mut self, handle: SlotMapHandle<T>) -> (T, Option<String>) {
        assert!(!self.handle_is_placeholder(&handle));

        let item_entry = &mut self.indirection_indices[handle.indirection_index];
        let item_index = item_entry.item_index;
        let path = item_entry.path.take();
        self.indirection_indices[*self.item_to_indirection_index.last().unwrap()].item_index =
            item_index;
        let removed_item = self.items.swap_remove(item_index);
        self.item_to_indirection_index.swap_remove(item_index);
        self.free_indirection_indices.push(handle.indirection_index);
        (removed_item, path)
    }

    pub fn get(&self, handle: &SlotMapHandle<T>) -> Option<&T> {
        self.items
            .get(self.indirection_indices[handle.indirection_index].item_index)
    }

    pub fn get_mut(&mut self, handle: &SlotMapHandle<T>) -> Option<&mut T> {
        self.items
            .get_mut(self.indirection_indices[handle.indirection_index].item_index)
    }
}
