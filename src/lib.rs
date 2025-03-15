use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowId};

mod logging;
mod windowing;

#[derive(Default, Debug)]
struct App {
    window: Option<Window>,
    // window: Option<Box<dyn Window>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        self.window = event_loop.create_window(window_attributes).ok();

        todo!()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        todo!()
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App::default();

    // For alternative loop run options see `pump_events` and `run_on_demand` examples.
    event_loop.run_app(&mut app).map_err(Into::into)
}
