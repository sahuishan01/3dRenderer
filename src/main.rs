pub mod application;
pub mod rendering;
pub mod utils;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut app = crate::application::app::App::default();
    let _ = event_loop.run_app(&mut app);
}
