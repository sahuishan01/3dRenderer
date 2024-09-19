pub mod utils;
pub mod rendering;
pub mod application;

use winit::event_loop::{ControlFlow, EventLoop};


fn main(){
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = crate::application::app::App::default();
    let _ = event_loop.run_app(&mut app);
}

