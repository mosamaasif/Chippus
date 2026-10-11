use imgui::{Context, StyleColor};

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

pub fn apply(imgui: &mut Context) {
    let style = imgui.style_mut();
    style.window_rounding = 8.0;
    style.scrollbar_rounding = 8.0;
    style.frame_rounding = 8.0;
    style[StyleColor::TitleBg] = RGBA::to_rgba_normalized([94, 69, 75, 62]);
    style[StyleColor::TitleBgCollapsed] = RGBA::to_rgba_normalized([94, 69, 75, 52]);
    style[StyleColor::TitleBgActive] = RGBA::to_rgba_normalized([94, 69, 75, 87]);
    style[StyleColor::Header] = RGBA::to_rgba_normalized([216, 179, 312, 52]);
    style[StyleColor::HeaderHovered] = RGBA::to_rgba_normalized([216, 179, 132, 92]);
    style[StyleColor::HeaderActive] = RGBA::to_rgba_normalized([216, 179, 132, 72]);
    style[StyleColor::ScrollbarBg] = RGBA::to_rgba_normalized([216, 179, 132, 12]);
    style[StyleColor::ScrollbarGrab] = RGBA::to_rgba_normalized([216, 179, 132, 52]);
    style[StyleColor::ScrollbarGrabHovered] = RGBA::to_rgba_normalized([216, 179, 312, 92]);
    style[StyleColor::ScrollbarGrabActive] = RGBA::to_rgba_normalized([216, 179, 312, 72]);
    style[StyleColor::SliderGrab] = RGBA::to_rgba_normalized([216, 179, 312, 52]);
    style[StyleColor::SliderGrabActive] = RGBA::to_rgba_normalized([216, 179, 312, 72]);
    style[StyleColor::Button] = RGBA::to_rgba_normalized([206, 229, 208, 30]);
    style[StyleColor::ButtonHovered] = RGBA::to_rgba_normalized([206, 229, 208, 100]);
    style[StyleColor::ButtonActive] = RGBA::to_rgba_normalized([206, 229, 208, 60]);
    style[StyleColor::PopupBg] = RGBA::to_rgba_normalized([0, 0, 0, 230]);
    style[StyleColor::TextSelectedBg] = RGBA::to_rgba_normalized([10, 23, 18, 180]);
    style[StyleColor::FrameBg] = RGBA::to_rgba_normalized([216, 179, 132, 30]);
    style[StyleColor::FrameBgHovered] = RGBA::to_rgba_normalized([216, 179, 132, 70]);
    style[StyleColor::FrameBgActive] = RGBA::to_rgba_normalized([216, 179, 132, 50]);
    style[StyleColor::MenuBarBg] = RGBA::to_rgba_normalized([216, 179, 132, 30]);
}
