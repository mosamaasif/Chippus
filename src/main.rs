use application::app::Application;

mod application;
mod emulator;
mod imgui_wgpu_backend;

use std::rc::Rc;

fn main() {
    let app = Rc::new(Application::new());
    app.run()
}
