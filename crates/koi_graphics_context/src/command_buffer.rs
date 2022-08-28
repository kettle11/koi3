use crate::*;

pub enum Command {
    Clear(kmath::Vec4),
    Present,
    SetPipeline(Pipeline),
    Draw {
        triangle_buffer: Option<TriangleBuffer>,
        start_triangle: u32,
        end_triangle: u32,
        instances: u32,
    },
}

pub struct CommandBuffer(pub(crate) Vec<Command>);

impl CommandBuffer {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, command: Command) {
        self.0.push(command);
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }
}
