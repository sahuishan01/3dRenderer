pub mod vector;
pub mod ray;
mod utils;

use vector::Vec3;

use slint::{ComponentHandle, RenderingState, Rgba8Pixel, SharedPixelBuffer, Weak};
use std::thread;
use slint::platform::WindowEvent;
use utils::{generate_gradient};

fn update_image(handle_weak: &Weak<MainWindow>, width: u32, height: u32){
    if width==0 || height==0 { return; };
    println!("updating");
    let handle_copy = handle_weak.clone();
    let aspect_ratio = width as f64 / height as f64;
    let focal_length: f64 = 1.;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * aspect_ratio;
    let camera_center: Vec3<f64> = Vec3::new(0., 0., 0.);

    let viewport_u = Vec3::new(viewport_width, 0., 0.);
    let viewport_v = Vec3::new(0., -viewport_height, 0.);

    let pixel_delta_u = viewport_u / width as f64;
    let pixel_delta_v = viewport_v / height as f64;

    let viewport_upper_left =
        camera_center + Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2. - viewport_v / 2.;

    let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;
    let th = thread::spawn(move || {
        let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
        generate_gradient(pixel_buffer.width(), pixel_buffer.make_mut_bytes(), pixel00_loc, pixel_delta_u, pixel_delta_v, camera_center);
        let _ = slint::invoke_from_event_loop(move || {
            handle_copy.upgrade().unwrap().set_canvas_source(slint::Image::from_rgba8_premultiplied(pixel_buffer));
        });
    });
    let _ = th.join();
    // slint::Image::from_rgba8(pixel_buffer)
}

fn process_rendering_state(state: RenderingState, main_window: MainWindow, width: &u32, height: &u32){
    match state {
        RenderingState::BeforeRendering => {
            update_image(&main_window.as_weak(), width.clone(), height.clone());
        }
        RenderingState::RenderingSetup => {
            update_image(&main_window.as_weak(), width.clone(), height.clone());
        }
        _ => {
            println!("{:?}", state);
        }
    }
}

fn main() {

    let mut width = 800;
    let mut height = 800;

    let main_window = MainWindow::new().unwrap();
    main_window.window().dispatch_event(WindowEvent::Resized { size: Default::default()});
    let handle_weak = main_window.clone_strong();
    let _ = main_window.window().set_rendering_notifier(move |state, _graphic| {
        if handle_weak.window().size().width != width || handle_weak.window().size().height != height{
            width = handle_weak.window().size().width.clone();
            height = handle_weak.window().size().height.clone();
            process_rendering_state(state, handle_weak.clone_strong(), &width, &height);
        }
    });
    main_window.run().unwrap();
}


slint::slint!{
    export component MainWindow inherits Window{
        in property <image> canvas_source <=> canvas.source;
        TouchArea {
            canvas := Image {
                height: parent.height;
                width: parent.width;
            }
        }
    }
}

