use winit::{
    dpi::{LogicalPosition, LogicalSize},
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::Window as WinitWindow,
};

pub struct WindowConfig {
    pub title: String,
    pub size: LogicalSize<f64>,
    pub position: LogicalPosition<f64>,
    pub resizable: bool,
}

/// What the event handler wants the window to do after an event.
pub enum Control {
    Continue,
    Exit,
}

pub struct Window {
    event_loop: EventLoop<()>,
    window: WinitWindow,
}

impl Window {
    pub fn new(config: WindowConfig) -> Window {
        let event_loop = EventLoop::new();

        let window = WinitWindow::new(&event_loop).unwrap();
        window.set_resizable(config.resizable);
        window.set_inner_size(config.size);
        window.set_title(&config.title);
        window.set_outer_position(config.position);

        Window { event_loop, window }
    }

    /// Native window handle, for backends that need to create surfaces or attach input.
    pub fn handle(&self) -> &WinitWindow {
        &self.window
    }

    /// Pumps OS events into `on_event` until it returns `Control::Exit`. Requests a redraw
    /// every iteration; what to do with each event is entirely up to the caller.
    pub fn run_event_loop<F>(self, mut on_event: F) -> !
    where
        F: 'static + FnMut(&WinitWindow, &Event<'_, ()>) -> Control,
    {
        let Window { event_loop, window } = self;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = if cfg!(feature = "metal-auto-capture") {
                ControlFlow::Exit
            } else {
                ControlFlow::Poll
            };

            if let Event::MainEventsCleared = event {
                window.request_redraw();
            }

            if let Control::Exit = on_event(&window, &event) {
                *control_flow = ControlFlow::Exit;
            }
        })
    }
}
