use crate::emulator::chip8;
use crate::render::backend::{AppCommand, AppView, FrameContext, RenderBackend};
use crate::render::imgui_wgpu::ImguiWgpuBackend;
use crate::window::input;
use crate::window::main_window::{Control, Window, WindowConfig};
use glob::glob;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
};

pub struct Application {
    emulator: chip8::Emulator,
    roms: Vec<PathBuf>,
}

impl Application {
    pub fn new() -> Application {
        Application {
            emulator: chip8::Emulator::new(),
            roms: Application::load_roms(),
        }
    }

    pub fn run(mut self) -> ! {
        let window = Window::new(WindowConfig {
            title: "CHIPPUS - CHIP8 EMU".to_string(),
            size: LogicalSize {
                width: 1398.0,
                height: 632.0,
            },
            //TODO: Should be dynamic
            position: LogicalPosition { x: 20.0, y: 100.0 },
            resizable: false,
        });

        // Swap the rendering backend here.
        let mut backend = ImguiWgpuBackend::new(window.handle());

        let mut commands = Vec::new();
        let mut last_frame = Instant::now();

        window.run_event_loop(move |native, event| {
            let mut control = Control::Continue;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    backend.resize(native.inner_size());
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    state: ElementState::Pressed,
                                    ..
                                },
                            ..
                        },
                    ..
                }
                | Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    control = Control::Exit;
                }
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode: Some(virtual_keycode),
                                    state,
                                    ..
                                },
                            ..
                        },
                    ..
                } => {
                    self.set_key(
                        input::map_key(*virtual_keycode),
                        *state == ElementState::Pressed,
                    );
                }
                Event::RedrawEventsCleared => {
                    let now = Instant::now();
                    let ctx = FrameContext {
                        dt: now - last_frame,
                    };
                    last_frame = now;

                    self.update(ctx.dt);
                    backend.render(native, &ctx, &self.view(), &mut commands);
                    self.frame_presented();
                    self.apply(commands.drain(..), ctx.dt);
                }
                _ => (),
            }

            backend.handle_event(native, event);

            control
        })
    }

    fn update(&mut self, dt: Duration) {
        self.emulator.execute_cycle(dt.as_secs_f32());
    }

    fn set_key(&mut self, key: usize, pressed: bool) {
        self.emulator.keyboard.set(key, pressed)
    }

    fn view(&self) -> AppView<'_> {
        AppView {
            emulator: &self.emulator,
            roms: &self.roms,
            framebuffer: &self.emulator.screen.buffer,
            framebuffer_dirty: self.emulator.screen.is_dirty(),
        }
    }

    /// Marks the current framebuffer as consumed by the backend.
    fn frame_presented(&mut self) {
        self.emulator.screen.set_dirty(false);
    }

    fn apply<I: IntoIterator<Item = AppCommand>>(&mut self, commands: I, dt: Duration) {
        for command in commands {
            match command {
                AppCommand::LoadRom(rom) => self.emulator.load_rom(&rom),
                AppCommand::Pause => self.emulator.pause = true,
                AppCommand::Resume => self.emulator.pause = false,
                AppCommand::Step => {
                    self.emulator.pause = false;
                    self.emulator.execute_cycle(dt.as_secs_f32());
                    self.emulator.pause = true;
                }
            }
        }
    }

    fn load_roms() -> Vec<PathBuf> {
        let executable_path = std::env::current_exe();
        let rom_path = executable_path
            .unwrap()
            .parent()
            .unwrap()
            .join("../../roms");

        glob(rom_path.join("**/*.ch8").to_str().unwrap())
            .unwrap()
            .map(|r| r.unwrap())
            .collect()
    }
}
