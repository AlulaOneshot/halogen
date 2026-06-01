use halogen_core::paint::PaintCommand;

#[cfg(feature = "skia")]
pub mod tiny_skia;

pub trait Backend {
    fn begin_frame(&mut self);
    fn execute(&mut self, commands: &[PaintCommand]);
    fn end_frame(&mut self); // raw RGBA pixels to render.
}