use application::app::Application;

mod application;
mod emulator;
mod render;
mod window;

fn main() {
    Application::new().run()
}
