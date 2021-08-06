use crate::emulator::chip8::Emulator;
use crate::emulator::screen::Screen;
use crate::imgui_wgpu_backend::Renderer;
use imgui::*;
use wgpu::{Device, Queue};

pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RGBA {
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn to_rgba_normalized(color: [i32; 4]) -> [f32; 4] {
        return [
            (color[0] as f32) / 255.0,
            (color[1] as f32) / 255.0,
            (color[2] as f32) / 255.0,
            (color[3] as f32) / 255.0,
        ];
    }
}

pub struct EmulatorWindow {
    data: Vec<u8>,
    width: usize,
    height: usize,
    scale: f32,
    color: RGBA,
    tex_id: TextureId,
}

impl EmulatorWindow {
    pub fn new(renderer: &mut Renderer, device: &Device) -> EmulatorWindow {
        EmulatorWindow {
            data: vec![0; Screen::WIDTH * Screen::HEIGHT * 4],
            width: Screen::WIDTH,
            height: Screen::HEIGHT,
            scale: 11.0f32,
            color: RGBA {
                r: 0.19f32,
                g: 0.66f32,
                b: 0.38f32,
                a: 1.0f32,
            },
            tex_id: renderer.create_texture(device, Screen::WIDTH as u32, Screen::HEIGHT as u32),
        }
    }

    pub fn render(&mut self, ui: &imgui::Ui, emulator: &mut Emulator, dt: f32) {
        let win = imgui::Window::new(im_str!("Emulator Window")).resizable(false);
        win.position([5.0f32, 5.0f32], imgui::Condition::Once)
            .build(&ui, || {
                Image::new(
                    self.tex_id,
                    [
                        (self.width as f32) * self.scale,
                        (self.height as f32) * self.scale,
                    ],
                )
                .tint_col(self.color.to_array())
                .build(&ui);

                if ui.button(im_str!("PAUSE"), [0f32, 0f32]) {
                    emulator.pause = true;
                }
                ui.same_line(0.0f32);
                if ui.button(im_str!("START"), [0f32, 0f32]) {
                    emulator.pause = false;
                }
                ui.same_line(0.0f32);
                if ui.button(im_str!("STEP"), [0f32, 0f32]) {
                    emulator.pause = false;
                    emulator.execute_cycle(dt);
                    emulator.pause = true;
                }

                ui.same_line(0.0f32);
                let mut color = self.color.to_array();
                imgui::ColorEdit::new(im_str!("Main Color"), &mut color).build(&ui);
                self.color = RGBA {
                    r: color[0],
                    g: color[1],
                    b: color[2],
                    a: color[3],
                };
            });
    }

    pub fn update(
        &mut self,
        emulator: &Emulator,
        renderer: &mut Renderer,
        device: &Device,
        mut queue: &mut Queue,
    ) {
        for x in 0..self.width {
            for y in 0..self.height {
                let v = if emulator.screen.get_pixel(x, y) == 1 {
                    255
                } else {
                    0
                };

                let pos = (y * 4 * self.width) + (x * 4);
                self.data[pos..pos + 4].copy_from_slice(&[v, v, v, 255]);
            }
        }

        // Uploaded updated screen texture data
        renderer.update_texture(
            self.tex_id,
            &device,
            &mut queue,
            &self.data,
            self.width as u32,
            self.height as u32,
        );
    }
}
