pub mod vector;
pub mod ray;
mod utils;
mod bvh;
mod mesh;

use bvh::BVH;
use ray::Camera;
use vector::Vec3;

use slint::private_unstable_api::re_exports::EventResult;

use slint::{quit_event_loop, run_event_loop, PhysicalSize, RenderingState, Rgba8Pixel, SharedPixelBuffer, Weak};
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
// use std::time::{Duration, Instant};
use std::path::PathBuf;
use std::time::Instant;
use slint::platform::WindowEvent;
// use walkdir::WalkDir;
use mesh::*;
use utils::generate_image;
use crate::bvh::create_bvh;


fn update_image(handle_weak: Weak<MainWindow>, width: &u32, height: &u32, camera: &Camera, bvh: Arc<Mutex<BVH>>){
    if *width==0 || *height==0 { return; };
    println!("updating {}, {}", width, height);
    let aspect_ratio = *width as f32 / *height as f32;
    let half_height = (camera.view_angle / 2.).tan() * camera.near;
    let half_width = half_height *  aspect_ratio;

    let pixel_width = 2. * half_width / *width as f32;
    let pixel_height = 2. * half_height / *height as f32;

    let z = (&camera.focus - &camera.position).normalize().convert();
    let x = camera.up.cross(&z).normalize().convert();
    let y: Vec3<f32> = z.cross(&x).normalize().convert();
    let view_port_center = &camera.position + z * camera.near;
    // let th = thread::spawn(move || {
        let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(*width, *height);
        let camera = camera.clone();
        // generate_gradient(pixel_buffer.width(), pixel_buffer.make_mut_bytes(), pixel00_loc, pixel_delta_u, pixel_delta_v, camera_center);
        let thread = std::thread::spawn(move || {
            let bvh = bvh.lock().unwrap();
            generate_image(
                pixel_buffer.width(),
                pixel_buffer.make_mut_bytes(),
                view_port_center,
                half_width,
                half_height,
                pixel_width,
                pixel_height,
                camera.position,
                &bvh.nodes,
                &bvh.triangles,
                x,
                y
            );
            let _res = handle_weak.upgrade_in_event_loop(move |handle| handle.set_canvas_source(slint::Image::from_rgba8(pixel_buffer)));
            println!("update finished");
        });
        _ = thread.join();
    // });
    // let _ = th.join();
    // slint::Image::from_rgba8(pixel_buffer)
}

fn process_rendering_state(state: RenderingState, main_window: Weak<MainWindow>, width: &u32, height: &u32, camera: &Camera, bvh: Arc<Mutex<BVH>>){
    match state {
        RenderingState::BeforeRendering => {
            update_image(main_window, width, height, camera, bvh);
        }
        RenderingState::RenderingSetup => {
            update_image(main_window, width, height, camera, bvh);
        }
        _ => {
            println!("{:?}", state);
        }
    }
}

fn load_mesh(path: &str) -> (Mesh, PathBuf) {
    let path = PathBuf::from_str(path).unwrap();
    let result = mesh::load_mesh(&path).unwrap();
    (result, path)
}



fn main() {

    let width = Arc::new(Mutex::new(800));
    let height = Arc::new(Mutex::new(800));

    let camera = Arc::new(Mutex::new(Camera::new(Some(Vec3::new(0., 0., -20.)), None,
                                     None, Some(0.01), None, Some(35.))));
    let cc1 = Arc::clone(&camera);
    let cc2 = Arc::clone(&camera);
    let cc3 = Arc::clone(&camera);

    let w1 = Arc::clone(&width);
    let w2 = Arc::clone(&width);
    let w3 = Arc::clone(&width);
    let h1 = Arc::clone(&height);
    let h2 = Arc::clone(&height);
    let h3 = Arc::clone(&height);

    let mesh: &'static _ = Box::leak(Box::new(load_mesh(r"/run/media/sahu/New Volume/projects/Raw/bunny.stl")));

    let start = Instant::now();
    let bvh = Arc::new(Mutex::new(create_bvh(&mesh.0, 40)));
    println!("Time taken for bvh: {}", (Instant::now() - start).as_secs_f32());

    let bvh_clone1 = Arc::clone(&bvh);
    let bvh_clone2 = Arc::clone(&bvh);
    let bvh_clone3 = Arc::clone(&bvh);

    let ctrl_pressed = Arc::new(Mutex::new(false));
    let ctrl_pressed_clone1 = Arc::clone(&ctrl_pressed);
    let ctrl_pressed_clone2 = Arc::clone(&ctrl_pressed);

    let main_window = MainWindow::new().unwrap();

    let size = PhysicalSize::new(*width.lock().unwrap(), *height.lock().unwrap());
    main_window.window().set_size(size);
    

    let handle_weak = main_window.as_weak();
    main_window.window().dispatch_event(WindowEvent::Resized { size: Default::default()});
    main_window.on_moved(move |pressed_x, pressed_y, current_x, current_y|{
        let dx = current_x - pressed_x;
        let dy = current_y  - pressed_y;
        let width = w1.lock().unwrap();
        let height = h1.lock().unwrap();
        let mut camera = cc1.lock().unwrap();
        let bvh= Arc::clone(&bvh_clone1);
        update_image(handle_weak.clone(), &width, &height, &camera, bvh);
        println!("x: {}, y: {}", pressed_x - current_x, pressed_y - current_y);
    });

    let handle_weak = main_window.as_weak();
    main_window.on_keyPressed(move |event| {
        let mut camera = cc3.lock().unwrap();
        let mut ctrl_pressed = ctrl_pressed_clone2.lock().unwrap();

        let width = w2.lock().unwrap();
        let height = h2.lock().unwrap();
        match event.text.to_ascii_lowercase().as_str(){
            "a" => camera.movement(ray::Direction::Left, &ctrl_pressed),
            "d" => camera.movement(ray::Direction::Right, &ctrl_pressed),
            "w" => camera.movement(ray::Direction::Up, &ctrl_pressed),
            "s" => camera.movement(ray::Direction::Down, &ctrl_pressed),
            "z" => camera.movement(ray::Direction::Forward, &ctrl_pressed),
            "x" => camera.movement(ray::Direction::Backward, &ctrl_pressed),
            "q" => {quit_event_loop();},
            "\u{11}" => {
                *ctrl_pressed = true;
                return EventResult::Accept;
            },
            _ => {}
        }
        let bvh= Arc::clone(&bvh_clone2);
        update_image(handle_weak.clone(), &width, &height, &camera, bvh);
        EventResult::Accept
    });

    main_window.on_keyReleased(move |event| {
        let mut ctrl_pressed = ctrl_pressed_clone1.lock().unwrap();
        match event.text.to_ascii_lowercase().as_str(){
            "\u{11}" => *ctrl_pressed = false,
            _ => {}
        }
        EventResult::Accept
    });

    let handle_weak = main_window.as_weak();
    let _ = main_window.window().set_rendering_notifier(move |state, _graphic| {
        let mut width = w3.lock().unwrap();
        let mut height = h3.lock().unwrap();
        let window_width = handle_weak.unwrap().window().size().width;
        let window_height = handle_weak.unwrap().window().size().height;
        if window_width != *width || window_height != *height{
            *width = window_width;
            *height = window_height;
            let camera = cc2.lock().unwrap();
            let bvh= Arc::clone(&bvh_clone3);
            process_rendering_state(state, handle_weak.clone(), &width, &height, &camera, bvh);
    }
    });


    main_window.run().unwrap();
}

slint::slint!{
    export component MainWindow inherits Window{
        in property <image> canvas_source <=> canvas.source;
        callback moved(length, length, length, length);
        callback keyPressed <=> keyboard.key-pressed;
        callback keyReleased <=> keyboard.key-released;
        // in-out property <function> name;
        keyboard:= FocusScope {
            touchArea:= TouchArea {
                canvas := Image {
                    height: parent.height;
                    width: parent.width;
                }
                moved => {root.moved(self.pressed-x, self.pressed-y, self.mouse-x, self.mouse-y)}
            }
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