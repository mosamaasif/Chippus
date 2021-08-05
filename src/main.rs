use application::app::Application;

mod application;
mod emulator;

use std::rc::Rc;

fn main() {
    let app = Rc::new(Application::new());
    app.run()
}
