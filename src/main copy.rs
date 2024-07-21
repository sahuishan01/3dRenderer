pub mod vector;
pub mod ray;
mod utils;
mod bvh;
mod mesh;

use bvh::is_on_box_boundary;
use ray::Camera;
use vector::Vec3;

use slint::{RenderingState, Rgba8Pixel, SharedPixelBuffer, Weak};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::thread;
use std::path::PathBuf;
use slint::platform::WindowEvent;
use walkdir::WalkDir;
use mesh::*;
use utils::{generate_gradient, generate_image};
use crate::bvh::create_bvh;
use crate::ray::Ray;


fn update_image(handle_weak: &Weak<MainWindow>, width: u32, height: u32, camera: &Camera){
    if width==0 || height==0 { return; };
    println!("updating");
    let handle_copy = handle_weak.clone();
    let aspect_ratio = width as f64 / height as f64;
    let viewport_height = 2.0;
    let viewport_width = viewport_height * aspect_ratio;

    let viewport_u = Vec3::new(viewport_width, 0., 0.);
    let viewport_v = Vec3::new(0., -viewport_height, 0.);

    let pixel_delta_u = viewport_u / width as f64;
    let pixel_delta_v = viewport_v / height as f64;

    let viewport_upper_left =
        camera.position + Vec3::new(0.0, 0.0, camera.near) - viewport_u / 2. - viewport_v / 2.;
    println!("{:?} {:?}",camera.position, camera.focus - camera.position);

    let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;
    let box_min = Vec3::new(-2.0, 2.0, -2.0);
    let box_max = Vec3::new(2.0, 6.0, 2.0);
    // let th = thread::spawn(move || {
        let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
        // generate_gradient(pixel_buffer.width(), pixel_buffer.make_mut_bytes(), pixel00_loc, pixel_delta_u, pixel_delta_v, camera_center);
        generate_image(
            pixel_buffer.width(),
            pixel_buffer.make_mut_bytes(),
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            camera.position,
            box_min,
            box_max
        );
        let _ = slint::invoke_from_event_loop(move || {
            handle_copy.upgrade().unwrap().set_canvas_source(slint::Image::from_rgba8_premultiplied(pixel_buffer));
        });
    // });
    // let _ = th.join();
    // slint::Image::from_rgba8(pixel_buffer)
}

fn process_rendering_state(state: RenderingState, main_window: MainWindow, width: &u32, height: &u32, camera: &Camera){
    match state {
        RenderingState::BeforeRendering => {
            update_image(&main_window.as_weak(), width.clone(), height.clone(), camera);
        }
        RenderingState::RenderingSetup => {
            update_image(&main_window.as_weak(), width.clone(), height.clone(), camera);
        }
        _ => {
            println!("{:?}", state);
        }
    }
}



fn main() {

    let mut width = 800;
    let mut height = 800;

    let camera = Arc::new(Mutex::new(Camera::new(Some(Vec3::new(0., 0., -10.)), None,
                                     None, Some(1.), None, None)));
    let camera_clone1 = Arc::clone(&camera);
    let camera_clone2 = Arc::clone(&camera);

    let main_window = MainWindow::new().unwrap();
    let handle_weak = main_window.clone_strong();
    main_window.window().dispatch_event(WindowEvent::Resized { size: Default::default()});
    main_window.on_moved(move |pressed_x, pressed_y, current_x, current_y|{
        let dx = current_x - pressed_x;
        let dy = current_y  - pressed_y;
        let mut camera = camera_clone1.lock().unwrap();
        camera.rotate(dx as f64, dy as f64, None);
        update_image(&handle_weak.as_weak(), width, height, &camera);
        println!("x: {}, y: {}", pressed_x - current_x, pressed_y - current_y);
    });
    let handle_weak = main_window.clone_strong();
    let _ = main_window.window().set_rendering_notifier(move |state, _graphic| {
        if handle_weak.window().size().width != width || handle_weak.window().size().height != height{
            width = handle_weak.window().size().width.clone();
            height = handle_weak.window().size().height.clone();
            let camera = camera_clone2.lock().unwrap();
            process_rendering_state(state, handle_weak.clone_strong(), &width, &height, &camera);
    }
    });
    main_window.run().unwrap();
}

slint::slint!{
    export component MainWindow inherits Window{
        in property <image> canvas_source <=> canvas.source;
        callback moved(length, length, length, length);
        // in-out property <function> name;
        touchArea:= TouchArea {
            canvas := Image {
                height: parent.height;
                width: parent.width;
            }
            moved => {root.moved(self.pressed-x, self.pressed-y, self.mouse-x, self.mouse-y)}
        }
    }
}

// fn main() -> io::Result<()>{
//
//     let start = std::time::Instant::now();
//
//     let mut meshes: Vec<(PathBuf, crate::mesh::Mesh)> = vec![];
//
//     let paths: Vec<PathBuf> = vec![PathBuf::from("../Raw/untitled.stl")];
//     // let mut paths: Vec<PathBuf> = WalkDir::new("../Raw")
//     //     .into_iter().filter_map(|entry | entry.ok())
//     //     .filter(|entry | entry.file_type().is_file())
//     //     .map(|entry | entry.path().to_path_buf())
//     //     .take(10)
//     //     .collect();
//     // let paths: Vec<_> = vec![PathBuf::from("../../../../Work/Mira/Raw/bunny_.stl")];
//     println!("{}", paths.len());
//     crate::mesh::add_meshes(&mut meshes, paths);
//
//
//     // for mesh in &meshes{
//     //     println!("For mesh {:?}, number of vertices is {:?}", mesh.0.file_name(), mesh.1.vertices.len())
//     // }
//
//     // paths = WalkDir::new("../../../Work/Mira/Packing_stls")
//     //     .into_iter().filter_map(|entry | entry.ok())
//     //     .filter(|entry | entry.file_type().is_file())
//     //     .map(|entry | entry.path().to_path_buf())
//     //     .collect();
//     //
//     // add_meshes(&mut meshes, paths);
//
//     crate::mesh::sort_meshes_by_num_faces(&mut meshes);
//     crate::mesh::process_meshes(&mut meshes, 0);
//
//
//     // for mesh in &meshes{
//     //     let mut save_path = "./".to_owned();
//     //     save_path.push_str(mesh.0.file_name().unwrap().to_str().unwrap());
//     //     // println!("{save_path}");
//     //     mesh.1.write_stl_file(save_path.as_str()).unwrap()
//     // }
//
//
//     // for mesh in &meshes{
//     //     println!("For mesh {:?}, number of vertices is {:?}", mesh.0.file_name(), mesh.1.vertices.len());
//     // }
//
//     // if meshes.len() > 0 {
//     //     let mesh = &meshes[meshes.len()-2].1;
//     //     println!("Number of triangles is {}", mesh.faces.len());
//     //     for vertex in &mesh.vertices[&mesh.vertices.len()-20..]{
//     //         println!("Vertex is: {:?}", vertex)
//     //     }
//     // }
//     println!("meshes size is: {}", meshes.len());
//
//     println!("Total time taken is: {} micro seconds", start.elapsed().as_micros() );
//
//     let ray = Ray::new(
//         Vec3::<f32>::e_new(), Vec3::<f32>::e_new()
//     );
//     let val = ray.at(20.);
//     println!("{:?}", val);
//
//     println!("{}", meshes[0].1.faces.len());
//     let start = Instant::now();
//     let _ = create_bvh(&meshes[0].1);
//     println!("Time for bvh: {}", (Instant::now() - start).as_secs_f32());
//
//     // let triangle_centers = create_bvh(&meshes[0].1);
//     // for center in &triangle_centers{
//     //     println!("{}", center);
//     // }
//
//     Ok(())
// }