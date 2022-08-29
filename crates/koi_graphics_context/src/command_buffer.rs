use crate::*;

pub enum Command {
    Clear(kmath::Vec4),
    Present,
    SetPipeline(Pipeline),
    Draw {
        triangle_buffer: Option<Buffer<[u32; 3]>>,
        triangle_range: std::ops::Range<u32>,
        instances: u32,
    },
}

pub struct CommandBuffer(pub(crate) Vec<Command>);

impl CommandBuffer {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, command: Command) {
        match &command {
            Command::Draw {
                triangle_buffer, ..
            } => {
                if let Some(triangle_buffer) = triangle_buffer {
                    assert_eq!(
                        triangle_buffer.handle.inner().buffer_usage,
                        BufferUsage::Index
                    )
                }
            }
            _ => {}
        }
        self.0.push(command);
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}
