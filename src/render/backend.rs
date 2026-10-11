use crate::emulator::chip8::Emulator;
use std::path::PathBuf;
use std::time::Duration;
use winit::{dpi::PhysicalSize, event::Event, window::Window};

/// Read-only snapshot of everything a backend may display.
pub struct AppView<'a> {
    pub emulator: &'a Emulator,
    pub roms: &'a [PathBuf],
    pub framebuffer: &'a [u8], // Screen::WIDTH * Screen::HEIGHT, one 0/1 value per pixel
    pub framebuffer_dirty: bool,
}

/// Intents raised by the UI. The application applies them after the frame,
/// so backends never mutate emulator state directly.
pub enum AppCommand {
    LoadRom(PathBuf),
    Pause,
    Resume,
    Step,
}

pub struct FrameContext {
    pub dt: Duration,
}

/// Everything the window needs from a UI/GPU stack. Backends are constructed by
/// the factory passed to `Window::run`, so each one can have its own constructor.
pub trait RenderBackend {
    /// Called for every winit event, so the backend can feed its own input handling.
    fn handle_event(&mut self, window: &Window, event: &Event<'_, ()>);

    fn resize(&mut self, size: PhysicalSize<u32>);

    /// Draws one frame. UI interactions are pushed into `commands`.
    fn render(
        &mut self,
        window: &Window,
        ctx: &FrameContext,
        view: &AppView,
        commands: &mut Vec<AppCommand>,
    );
}
