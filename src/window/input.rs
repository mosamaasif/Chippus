use winit::event::VirtualKeyCode;

/// Maps a physical key to its CHIP-8 keypad index.
pub fn map_key(code: VirtualKeyCode) -> usize {
    match code {
        VirtualKeyCode::Key1 => 0,
        VirtualKeyCode::Key2 => 1,
        VirtualKeyCode::Key3 => 2,
        VirtualKeyCode::Key4 => 3,
        VirtualKeyCode::Q => 4,
        VirtualKeyCode::W => 5,
        VirtualKeyCode::E => 6,
        VirtualKeyCode::R => 7,
        VirtualKeyCode::A => 8,
        VirtualKeyCode::S => 9,
        VirtualKeyCode::D => 10,
        VirtualKeyCode::F => 11,
        VirtualKeyCode::Z => 12,
        VirtualKeyCode::X => 13,
        VirtualKeyCode::C => 14,
        VirtualKeyCode::V => 15,
        _ => usize::default(),
    }
}
