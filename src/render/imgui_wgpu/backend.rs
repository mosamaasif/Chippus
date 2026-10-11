use super::gpu::Gpu;
use super::renderer::{Renderer, RendererConfig, Texture, TextureConfig};
use super::style;
use super::ui::{self, UiState};
use crate::emulator::screen::Screen;
use crate::render::backend::{AppCommand, AppView, FrameContext, RenderBackend};
use imgui::{FontSource, MouseCursor, TextureId};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use wgpu::{Extent3d, TextureFormat, TextureUsage};
use winit::{dpi::PhysicalSize, event::Event, window::Window};

/// Dear imgui UI drawn with wgpu.
pub struct ImguiWgpuBackend {
    gpu: Gpu,
    imgui: imgui::Context,
    platform: WinitPlatform,
    renderer: Renderer,
    clear_color: wgpu::Color,
    screen_texture: TextureId,
    screen_pixels: Vec<u8>,
    ui_state: UiState,
    last_cursor: Option<Option<MouseCursor>>,
}

impl ImguiWgpuBackend {
    pub fn new(window: &Window) -> ImguiWgpuBackend {
        let gpu = Gpu::new(window);

        let hidpi_factor = 1.0;

        // Set up dear imgui
        let mut imgui = imgui::Context::create();
        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Default);
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

        style::apply(&mut imgui);

        // Set up dear imgui wgpu renderer
        let renderer_config = RendererConfig {
            texture_format: Gpu::FORMAT,
            ..Default::default()
        };

        let mut renderer = Renderer::new(&mut imgui, &gpu.device, &gpu.queue, renderer_config);

        let screen_texture = create_screen_texture(&mut renderer, &gpu);

        ImguiWgpuBackend {
            gpu,
            imgui,
            platform,
            renderer,
            clear_color: wgpu::Color {
                r: 0.03,
                g: 0.03,
                b: 0.03,
                a: 1.0,
            },
            screen_texture,
            screen_pixels: vec![0; Screen::WIDTH * Screen::HEIGHT * 4],
            ui_state: UiState::new(),
            last_cursor: None,
        }
    }

    /// Expands the 1-bit framebuffer to RGBA and uploads it to the screen texture.
    fn upload_screen(&mut self, framebuffer: &[u8]) {
        let (rgba_pixels, _) = self.screen_pixels.as_chunks_mut::<4>();
        for (pixel, rgba) in framebuffer.iter().zip(rgba_pixels) {
            let v = if *pixel == 1 { 0xFF } else { 0 };
            *rgba = [v, v, v, 0xFF];
        }

        if let Some(texture) = self.renderer.textures.get(self.screen_texture) {
            texture.write(
                &self.gpu.queue,
                &self.screen_pixels,
                Screen::WIDTH as u32,
                Screen::HEIGHT as u32,
            );
        }
    }
}

impl RenderBackend for ImguiWgpuBackend {
    fn handle_event(&mut self, window: &Window, event: &Event<'_, ()>) {
        self.platform.handle_event(self.imgui.io_mut(), window, event);
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size);
    }

    fn render(
        &mut self,
        window: &Window,
        ctx: &FrameContext,
        view: &AppView,
        commands: &mut Vec<AppCommand>,
    ) {
        self.imgui.io_mut().update_delta_time(ctx.dt);

        // Upload before acquiring the frame so a dropped frame never loses a screen update
        if view.framebuffer_dirty {
            self.upload_screen(view.framebuffer);
        }

        let frame = match self.gpu.current_frame() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("dropped frame: {:?}", e);
                return;
            }
        };
        self.platform
            .prepare_frame(self.imgui.io_mut(), window)
            .expect("Failed to prepare frame");
        let ui = self.imgui.frame();

        ui::draw(&ui, view, &mut self.ui_state, self.screen_texture, commands);

        let mut encoder: wgpu::CommandEncoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        if self.last_cursor != Some(ui.mouse_cursor()) {
            self.last_cursor = Some(ui.mouse_cursor());
            self.platform.prepare_render(&ui, window);
        }

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.renderer
            .render(ui.render(), &self.gpu.queue, &self.gpu.device, &mut rpass)
            .expect("Rendering failed");

        drop(rpass);

        self.gpu.queue.submit(Some(encoder.finish()));
    }
}

fn create_screen_texture(renderer: &mut Renderer, gpu: &Gpu) -> TextureId {
    let texture_config = TextureConfig {
        label: None,
        size: Extent3d {
            width: Screen::WIDTH as u32,
            height: Screen::HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Some(TextureFormat::Rgba8Unorm),
        usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
    };

    let texture = Texture::new(&gpu.device, renderer, texture_config);

    renderer.textures.insert(texture)
}
