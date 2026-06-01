use halogen_core::paint::PaintCommand;
use halogen_core::widget::PaintSink;
use crate::backend::Backend;

pub struct Executor<B: Backend> {
    backend: B,
    commands: Vec<PaintCommand>,
}

impl<B: Backend> PaintSink for Executor<B> {
    fn push(&mut self, command: PaintCommand) {
        self.commands.push(command);
    }
}

impl<B: Backend> Executor<B> {
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            commands: Vec::new(),
        }
    }

    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    pub fn flush(&mut self,) {
        self.backend.begin_frame();
        self.backend.execute(&self.commands);
        self.commands.clear();
        self.backend.end_frame();
        // end_frame / present handled by platform
    }
}