use super::emu_window::{self, EmulatorWindow};
use crate::emulator::{chip8, keyboard::Keyboard};
use crate::imgui_wgpu_backend::{Renderer, RendererConfig};
use emu_window::RGBA;
use futures::executor::block_on;
use glob::glob;
use imgui::*;
use imgui_winit_support;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;
use wgpu::Instance;
use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
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

    pub fn render(&mut self, ui: &imgui::Ui) {
        // Window with list of ROMs
        let win = imgui::Window::new(im_str!("ROMs Available"));
        win.size([363.0f32, 623.0f32], Condition::Once)
            .position([1031.0f32, 5.0f32], Condition::Once)
            .resizable(false)
            .build(&ui, || {
                for rom in &self.roms {
                    let filename = ImString::new(rom.file_name().unwrap().to_str().unwrap());
                    if ui.button(&filename, [333.0f32, 0f32]) {
                        self.emulator.load_rom(rom);
                    }
                }
            });

        // Window with current CPU State
        let window = imgui::Window::new(im_str!("Current CPU State"));
        window
            .size([300.0f32, 210.0f32], Condition::FirstUseEver)
            .position([728.0f32, 418.0f32], Condition::Once)
            .resizable(false)
            .build(&ui, || {
                ui.text(format!("PC: {:#X}", self.emulator.pc));
                ui.text(format!("I: {:#X}", self.emulator.i));
                for i in 0..self.emulator.v.len() {
                    ui.text(format!("V{:X}: {:#X} ", i, self.emulator.v[i]));
                    if (i + 1) % 4 != 0 {
                        ui.same_line(0.0f32);
                    }
                }
                ui.text(format!("Delay Timer: {}", self.emulator.delay_timer));
                ui.text(format!("Sound Timer: {}", self.emulator.sound_timer));

                ui.text(format!(
                    "Stack:\n(Size: {}),\nValues:",
                    self.emulator.stack.len()
                ));
                for v in self.emulator.stack.iter() {
                    ui.text(format!("{:X}", v));
                    ui.same_line(0.0);
                }
            });

        // Window with program code
        let window = imgui::Window::new(im_str!("Code"));
        window
            .size([300.0, 410.0], Condition::FirstUseEver)
            .position([728.0, 5.0], Condition::Once)
            .resizable(false)
            .build(&ui, || {
                let code_location = self.emulator.code_memory_location();
                let pc = self.emulator.pc as usize;
                let code = &self.emulator.ram[code_location.0..code_location.1];
                for i in (1..code.len()).step_by(2) {
                    let mut color_stack: Option<ColorStackToken> = None;
                    if pc == (i + code_location.0 - 1) {
                        ui.set_scroll_here_y();
                        color_stack = Some(ui.push_style_color(
                            StyleColor::Text,
                            RGBA::to_rgba_normalized([0, 255, 0, 255]),
                        ));
                    }
                    ui.text(format!("{:>4}: {:02X}{:02X}", i, code[i - 1], code[i]));
                    if let Some(c) = color_stack {
                        c.pop(&ui);
                    }
                }
            });

        // Help Window
        let window = imgui::Window::new(im_str!("About"));
        window
            .size([720.0, 210.0], Condition::FirstUseEver)
            .position([5.0, 418.0], Condition::Once)
            .resizable(false)
            .build(&ui, || {
                ui.text(im_str!("Welcome to CHIPPUS! Yet another Chip8 Emulator written by a noob learning Rust!"));
                ui.text(im_str!("\nHow to use this Emulator?"));
                ui.text(im_str!("Step - 1:\n\tSelect ROM file."));
                ui.text(im_str!("\nStep - 2:\n\tUse these Controls:\n\t1,2,3,4,\n\tQ,W,E,R,\n\tA,S,D,F,\n\tZ,X,C,V"));
            });
    }

    fn style_ui(&self, imgui: &mut Context) {
        let style = imgui.style_mut();
        style.window_rounding = 8.0;
        style.scrollbar_rounding = 8.0;
        style.frame_rounding = 8.0;
        style[imgui::StyleColor::TitleBg] = RGBA::to_rgba_normalized([94, 69, 75, 62]);
        style[imgui::StyleColor::TitleBgCollapsed] = RGBA::to_rgba_normalized([94, 69, 75, 52]);
        style[imgui::StyleColor::TitleBgActive] = RGBA::to_rgba_normalized([94, 69, 75, 87]);
        style[imgui::StyleColor::Header] = RGBA::to_rgba_normalized([216, 179, 312, 52]);
        style[imgui::StyleColor::HeaderHovered] = RGBA::to_rgba_normalized([216, 179, 132, 92]);
        style[imgui::StyleColor::HeaderActive] = RGBA::to_rgba_normalized([216, 179, 132, 72]);
        style[imgui::StyleColor::ScrollbarBg] = RGBA::to_rgba_normalized([216, 179, 132, 12]);
        style[imgui::StyleColor::ScrollbarGrab] = RGBA::to_rgba_normalized([216, 179, 132, 52]);
        style[imgui::StyleColor::ScrollbarGrabHovered] =
            RGBA::to_rgba_normalized([216, 179, 312, 92]);
        style[imgui::StyleColor::ScrollbarGrabActive] =
            RGBA::to_rgba_normalized([216, 179, 312, 72]);
        style[imgui::StyleColor::SliderGrab] = RGBA::to_rgba_normalized([216, 179, 312, 52]);
        style[imgui::StyleColor::SliderGrabActive] = RGBA::to_rgba_normalized([216, 179, 312, 72]);
        style[imgui::StyleColor::Button] = RGBA::to_rgba_normalized([206, 229, 208, 30]);
        style[imgui::StyleColor::ButtonHovered] = RGBA::to_rgba_normalized([206, 229, 208, 100]);
        style[imgui::StyleColor::ButtonActive] = RGBA::to_rgba_normalized([206, 229, 208, 60]);
        style[imgui::StyleColor::PopupBg] = RGBA::to_rgba_normalized([0, 0, 0, 230]);
        style[imgui::StyleColor::TextSelectedBg] = RGBA::to_rgba_normalized([10, 23, 18, 180]);
        style[imgui::StyleColor::FrameBg] = RGBA::to_rgba_normalized([216, 179, 132, 30]);
        style[imgui::StyleColor::FrameBgHovered] = RGBA::to_rgba_normalized([216, 179, 132, 70]);
        style[imgui::StyleColor::FrameBgActive] = RGBA::to_rgba_normalized([216, 179, 132, 50]);
        style[imgui::StyleColor::MenuBarBg] = RGBA::to_rgba_normalized([216, 179, 132, 30]);
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

    fn set_key_state(&mut self, code: VirtualKeyCode, state: bool) {
        self.emulator.keyboard.set(Keyboard::map_key(code), state)
    }

    pub fn run(mut self: Rc<Self>) {
        // Set up window and GPU
        let event_loop = EventLoop::new();

        let instance = Instance::new(wgpu::BackendBit::PRIMARY);

        let (window, size, surface) = {
            let window = Window::new(&event_loop).unwrap();
            window.set_resizable(false);
            window.set_inner_size(LogicalSize {
                width: 1398.0,
                height: 632.0,
            });
            window.set_title("CHIPPUS - CHIP8 EMU");
            //TODO: Should be dynamic
            window.set_outer_position(LogicalPosition { x: 20.0, y: 100.0 });
            let size = window.inner_size();

            let surface = unsafe { instance.create_surface(&window) };

            (window, size, surface)
        };

        let hidpi_factor = 1.0;

        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, mut queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

        // Set up swap chain
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width as u32,
            height: size.height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };

        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // Set up dear imgui
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        self.style_ui(&mut imgui);

        // Setup dear imgui wgpu renderer
        let clear_color = wgpu::Color {
            r: 0.03,
            g: 0.03,
            b: 0.03,
            a: 1.0,
        };

        let renderer_config = RendererConfig {
            texture_format: sc_desc.format,
            ..Default::default()
        };

        let mut renderer = Renderer::new(&mut imgui, &device, &mut queue, renderer_config);

        let mut last_frame = Instant::now();

        let mut screen = EmulatorWindow::new(&mut renderer, &device);

        let mut last_cursor = None;

        // Event loop
        event_loop.run(move |event, _, control_flow| {
            let self_mut = Rc::get_mut(&mut self).unwrap();

            *control_flow = if cfg!(feature = "metal-auto-capture") {
                ControlFlow::Exit
            } else {
                ControlFlow::Poll
            };
            match event {
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    let size = window.inner_size();

                    let sc_desc = wgpu::SwapChainDescriptor {
                        usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        width: size.width as u32,
                        height: size.height as u32,
                        present_mode: wgpu::PresentMode::Mailbox,
                    };

                    swap_chain = device.create_swap_chain(&surface, &sc_desc);
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
                    *control_flow = ControlFlow::Exit;
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
                    self_mut.set_key_state(virtual_keycode, state == ElementState::Pressed);
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawEventsCleared => {
                    //let _delta_s = last_frame.elapsed();
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;

                    let frame = match swap_chain.get_current_frame() {
                        Ok(frame) => frame,
                        Err(e) => {
                            eprintln!("dropped frame: {:?}", e);
                            return;
                        }
                    };
                    platform
                        .prepare_frame(imgui.io_mut(), &window)
                        .expect("Failed to prepare frame");
                    let ui = imgui.frame();

                    // Run emulator update
                    self_mut.emulator.execute_cycle(ui.io().delta_time);

                    // Read and update screen buffer if changed:
                    if self_mut.emulator.screen.is_dirty() {
                        self_mut.emulator.screen.set_dirty(false);

                        screen.update(&self_mut.emulator, &mut renderer, &device, &mut queue);
                    }

                    // Draw actual app UI
                    self_mut.render(&ui);
                    // Draw screen window
                    screen.render(&ui, &mut self_mut.emulator, ui.io().delta_time);

                    let mut encoder: wgpu::CommandEncoder = device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                    if last_cursor != Some(ui.mouse_cursor()) {
                        last_cursor = Some(ui.mouse_cursor());
                        platform.prepare_render(&ui, &window);
                    }

                    let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &frame.output.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(clear_color),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });

                    renderer
                        .render(ui.render(), &queue, &device, &mut rpass)
                        .expect("Rendering failed");

                    drop(rpass);

                    queue.submit(Some(encoder.finish()));
                }
                _ => (),
            }

            platform.handle_event(imgui.io_mut(), &window, &event);
        });
    }
}
