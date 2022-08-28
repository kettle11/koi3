pub struct Assets<T: Clone> {
    drop_channel_send: std::sync::mpsc::Sender<T>,
    drop_channel_receive: std::sync::mpsc::Receiver<T>,
}

impl<T: Clone> Assets<T> {
    pub fn new() -> Self {
        let (drop_channel_send, drop_channel_receive) = std::sync::mpsc::channel();
        Self {
            drop_channel_send,
            drop_channel_receive,
        }
    }

    pub fn get_dropped_assets(&mut self) -> impl Iterator<Item = T> + '_ {
        self.drop_channel_receive.try_iter()
    }

    pub fn new_handle(&mut self, asset: T) -> Handle<T> {
        Handle {
            drop_handle: std::sync::Arc::new(DropHandle {
                t: asset,
                drop_channel: self.drop_channel_send.clone(),
            }),
        }
    }
}

#[derive(Clone)]
pub struct Handle<T: Clone> {
    drop_handle: std::sync::Arc<DropHandle<T>>,
}

impl<T: Clone> Handle<T> {
    pub fn inner(&self) -> &T {
        &self.drop_handle.t
    }
}

#[derive(Clone)]
pub struct DropHandle<T: Clone> {
    t: T,
    drop_channel: std::sync::mpsc::Sender<T>,
}

impl<T: Clone> Drop for DropHandle<T> {
    fn drop(&mut self) {
        let _ = self.drop_channel.send(self.t.clone());
    }
}
