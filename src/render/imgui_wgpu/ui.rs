use super::style::RGBA;
use crate::emulator::screen::Screen;
use crate::render::backend::{AppCommand, AppView};
use imgui::*;

/// Display preferences owned by this backend, not by the application.
pub struct UiState {
    pub screen_scale: f32,
    pub screen_color: RGBA,
}

impl UiState {
    pub fn new() -> UiState {
        UiState {
            screen_scale: 11.0f32,
            screen_color: RGBA {
                r: 0.0f32,
                g: 0.76f32,
                b: 0.02f32,
                a: 1.0f32,
            },
        }
    }
}

pub fn draw(
    ui: &Ui,
    view: &AppView,
    state: &mut UiState,
    screen_texture: TextureId,
    commands: &mut Vec<AppCommand>,
) {
    draw_roms(ui, view, commands);
    draw_cpu_state(ui, view);
    draw_code(ui, view);
    draw_about(ui);
    draw_screen(ui, state, screen_texture, commands);
}

fn draw_screen(
    ui: &Ui,
    state: &mut UiState,
    screen_texture: TextureId,
    commands: &mut Vec<AppCommand>,
) {
    let win = imgui::Window::new(im_str!("Emulator Window")).resizable(false);
    win.position([5.0f32, 5.0f32], imgui::Condition::Once)
        .build(&ui, || {
            Image::new(
                screen_texture,
                [
                    (Screen::WIDTH as f32) * state.screen_scale,
                    (Screen::HEIGHT as f32) * state.screen_scale,
                ],
            )
            .tint_col(state.screen_color.to_array())
            .build(&ui);

            if ui.button(im_str!("PAUSE"), [0f32, 0f32]) {
                commands.push(AppCommand::Pause);
            }
            ui.same_line(0.0f32);
            if ui.button(im_str!("START"), [0f32, 0f32]) {
                commands.push(AppCommand::Resume);
            }
            ui.same_line(0.0f32);
            if ui.button(im_str!("STEP"), [0f32, 0f32]) {
                commands.push(AppCommand::Step);
            }

            ui.same_line(0.0f32);
            let mut color = state.screen_color.to_array();
            imgui::ColorEdit::new(im_str!("Main Color"), &mut color).build(&ui);
            state.screen_color = RGBA {
                r: color[0],
                g: color[1],
                b: color[2],
                a: color[3],
            };
        });
}

fn draw_roms(ui: &Ui, view: &AppView, commands: &mut Vec<AppCommand>) {
    // Window with list of ROMs
    let win = imgui::Window::new(im_str!("ROMs Available"));
    win.size([363.0f32, 623.0f32], Condition::Once)
        .position([1031.0f32, 5.0f32], Condition::Once)
        .resizable(false)
        .build(&ui, || {
            for rom in view.roms {
                let filename = ImString::new(rom.file_name().unwrap().to_str().unwrap());
                if ui.button(&filename, [333.0f32, 0f32]) {
                    commands.push(AppCommand::LoadRom(rom.clone()));
                }
            }
        });
}

fn draw_cpu_state(ui: &Ui, view: &AppView) {
    let emulator = view.emulator;

    // Window with current CPU State
    let window = imgui::Window::new(im_str!("Current CPU State"));
    window
        .size([300.0f32, 210.0f32], Condition::FirstUseEver)
        .position([728.0f32, 418.0f32], Condition::Once)
        .resizable(false)
        .build(&ui, || {
            ui.text(format!("PC: {:#X}", emulator.pc));
            ui.text(format!("I: {:#X}", emulator.i));
            for i in 0..emulator.v.len() {
                ui.text(format!("V{:X}: {:#X} ", i, emulator.v[i]));
                if (i + 1) % 4 != 0 {
                    ui.same_line(0.0f32);
                }
            }
            ui.text(format!("Delay Timer: {}", emulator.delay_timer));
            ui.text(format!("Sound Timer: {}", emulator.sound_timer));

            ui.text(format!(
                "Stack:\n(Size: {}),\nValues:",
                emulator.stack.len()
            ));
            for v in emulator.stack.iter() {
                ui.text(format!("{:X}", v));
                ui.same_line(0.0);
            }
        });
}

fn draw_code(ui: &Ui, view: &AppView) {
    let emulator = view.emulator;

    // Window with program code
    let window = imgui::Window::new(im_str!("Code"));
    window
        .size([300.0, 410.0], Condition::FirstUseEver)
        .position([728.0, 5.0], Condition::Once)
        .resizable(false)
        .build(&ui, || {
            let code_location = emulator.code_memory_location();
            let pc = emulator.pc as usize;
            let code = &emulator.ram[code_location.0..code_location.1];
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
}

fn draw_about(ui: &Ui) {
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
